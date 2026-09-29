//! One public connection model for browser accounts and every enrolled bridge.

use super::connector_oauth::{Provider, VerificationReport};
use crate::error::{AdminError, AdminResult};
use crate::repositories::users::{connector_accounts as repo, queries};
use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt::models::auth::Permission;

pub(crate) use super::connector_readiness::probe_session_connection;

#[derive(Debug, Clone, Serialize)]
pub struct Connection {
    pub provider: String,
    pub display_name: String,
    pub requires_auth: bool,
    // Why: the connection is the caller's own signed-in session checked
    // against the server's `oauth.scopes`, not a grant they can connect or
    // disconnect. The card and the bridge render it as a live check.
    pub session_attested: bool,
    pub configured: bool,
    pub entitled: bool,
    pub status: String,
    pub auth_method: Option<String>,
    pub account_id: Option<String>,
    pub account_name: Option<String>,
    pub resource_id: Option<String>,
    pub resource_name: Option<String>,
    pub error_code: Option<String>,
    pub verified_at: Option<String>,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionSnapshot {
    pub version: u32,
    pub user_id: UserId,
    pub revision: i64,
    pub connections: Vec<Connection>,
    // Why: only a probe response carries a report; the bridge reads the same
    // snapshot and ignores the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<VerificationReport>,
}

pub(super) struct Entitlement {
    pub(super) active: bool,
    pub(super) roles: Vec<String>,
}

// Why: `oauth.scopes` on a service is the permission tier its calls demand;
// the admin tier is the manage roles, the user tier is any active account.
pub(super) fn scopes_granted(roles: &[String], scopes: &[Permission]) -> bool {
    scopes.iter().all(|scope| match scope {
        Permission::Admin => crate::types::roles_grant_manage(roles),
        Permission::User | Permission::Anonymous => true,
        _ => false,
    })
}

pub(super) async fn load_entitlement(pool: &PgPool, user: &UserId) -> AdminResult<Entitlement> {
    let identity = queries::find_identity_envelope(pool, user)
        .await?
        .ok_or_else(|| AdminError::Unauthorized("Account unavailable".into()))?;
    Ok(Entitlement {
        active: identity.status == "active",
        roles: identity.roles,
    })
}

impl Entitlement {
    // Why: a session-attested server lists the scopes; every other provider
    // is open to any active account.
    fn permits(&self, provider: &Provider) -> bool {
        self.active
            && provider
                .session_scopes()
                .is_none_or(|scopes| scopes_granted(&self.roles, scopes))
    }
}

fn not_entitled_message(provider: &Provider) -> &'static str {
    if provider.is_session_attested() {
        "This connector is not open to your account"
    } else {
        "An active account is required to connect providers"
    }
}

pub(crate) async fn require_entitlement(
    pool: &PgPool,
    user: &UserId,
    provider: Provider,
) -> AdminResult<()> {
    let entitlement = load_entitlement(pool, user).await?;
    if !entitlement.active {
        return Err(AdminError::Forbidden(
            "An active account is required to connect providers".into(),
        ));
    }
    if !entitlement.permits(&provider) {
        return Err(AdminError::Forbidden(
            not_entitled_message(&provider).into(),
        ));
    }
    Ok(())
}

fn actions_for(
    provider: &Provider,
    row: Option<&repo::ProviderConnection>,
    status: &str,
    configured: bool,
    entitled: bool,
) -> Vec<String> {
    let mut actions = Vec::new();
    if configured && entitled && provider.is_session_attested() {
        actions.push("test".into());
    } else if configured && entitled && provider.requires_auth() {
        actions.push(
            if status == "connected" {
                "reconnect"
            } else {
                "connect"
            }
            .into(),
        );
        if row.is_some_and(|r| r.auth_method.is_some()) {
            actions.push("test".into());
        }
        if matches!(provider, Provider::Atlassian | Provider::Github) {
            actions.push("manual_token".into());
        }
    }
    if row.is_some_and(|r| r.auth_method.is_some()) {
        actions.push("disconnect".into());
    }
    actions
}

fn describe(
    provider: &Provider,
    row: Option<&repo::ProviderConnection>,
    entitlement: &Entitlement,
) -> Connection {
    let configured = provider.configured();
    let entitled = entitlement.permits(provider);
    let session_attested = provider.is_session_attested();
    let status = if configured && !provider.requires_auth() {
        "no_auth_required"
    } else if configured && session_attested {
        if entitled {
            "connected"
        } else {
            "not_connected"
        }
    } else if configured {
        row.map_or("not_connected", |r| r.status.as_str())
    } else {
        "not_configured"
    };
    Connection {
        provider: provider.slug().into(),
        display_name: provider.display_name(),
        requires_auth: provider.requires_auth(),
        session_attested,
        configured,
        entitled,
        status: status.into(),
        auth_method: if session_attested {
            Some("session".into())
        } else {
            row.and_then(|r| r.auth_method.clone())
        },
        account_id: row.and_then(|r| r.account_id.clone()),
        account_name: row.and_then(|r| r.account_name.clone()),
        resource_id: row.and_then(|r| r.resource_id.clone()),
        resource_name: row.and_then(|r| r.resource_name.clone()),
        error_code: row.and_then(|r| r.error_code.clone()),
        // Why: the attestation is this request's own session check, so its
        // verification time is now; the bridge and the readiness gate read
        // `verified_at` alike for every connector.
        verified_at: if session_attested && entitled {
            Some(Utc::now().to_rfc3339())
        } else {
            row.and_then(|r| r.verified_at.map(|t| t.to_rfc3339()))
        },
        actions: actions_for(provider, row, status, configured, entitled),
    }
}

pub async fn get_connections(pool: &PgPool, user: &UserId) -> AdminResult<ConnectionSnapshot> {
    let rows = repo::list_accounts(pool, user).await?;
    let entitlement = load_entitlement(pool, user).await?;
    let services = systemprompt::loader::ServicesBootstrap::get().map_err(AdminError::internal)?;
    let mut ids = services
        .mcp_servers
        .iter()
        .filter(|(_, s)| s.enabled)
        .map(|(id, _)| id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    ids.extend(
        rows.iter()
            .filter(|r| r.auth_method.is_some())
            .map(|r| r.provider.clone()),
    );
    let mut connections = Vec::new();
    for id in ids {
        let provider = Provider::try_from(id).map_err(AdminError::BadRequest)?;
        let row = rows.iter().find(|r| r.provider == provider.slug());
        connections.push(describe(&provider, row, &entitlement));
    }
    Ok(ConnectionSnapshot {
        version: 1,
        user_id: user.clone(),
        revision: rows.iter().map(|r| r.revision).max().unwrap_or(0),
        connections,
        verification: None,
    })
}
