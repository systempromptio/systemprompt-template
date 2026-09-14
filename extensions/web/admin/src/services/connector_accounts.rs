//! One public connection model for browser accounts and every enrolled bridge.

use super::connector_oauth::Provider;
use crate::error::{AdminError, AdminResult};
use crate::repositories::groups::members;
use crate::repositories::users::{connector_accounts as repo, queries};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Connection {
    pub provider: String,
    pub display_name: String,
    pub requires_auth: bool,
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
pub(crate) struct ConnectionSnapshot {
    pub version: u32,
    pub user_id: UserId,
    pub revision: i64,
    pub connections: Vec<Connection>,
}

struct Entitlement {
    active: bool,
    groups: Vec<String>,
}

async fn load_entitlement(pool: &PgPool, user: &UserId) -> AdminResult<Entitlement> {
    let identity = queries::find_identity_envelope(pool, user)
        .await?
        .ok_or_else(|| AdminError::Unauthorized("Account unavailable".into()))?;
    let groups = members::list_group_ids_for_user(pool, user)
        .await?
        .into_iter()
        .map(|id| id.as_str().to_owned())
        .collect();
    Ok(Entitlement {
        active: identity.status == "active",
        groups,
    })
}

impl Entitlement {
    // Why: a Salesforce org lists the groups allowed to connect to it; every
    // other provider is open to any active account.
    fn permits(&self, provider: &Provider) -> bool {
        self.active
            && match provider {
                Provider::Salesforce(_) => provider
                    .salesforce_org()
                    .is_ok_and(|org| org.is_entitled(&self.groups)),
                _ => true,
            }
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
            "Your account is not entitled to this Salesforce org".into(),
        ));
    }
    Ok(())
}

pub(crate) async fn get_connections(
    pool: &PgPool,
    user: &UserId,
) -> AdminResult<ConnectionSnapshot> {
    let rows = repo::list_accounts(pool, user).await?;
    let entitlement = load_entitlement(pool, user).await?;
    let mut connections = Vec::new();
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
    for id in ids {
        let provider = Provider::try_from(id).map_err(AdminError::BadRequest)?;
        let row = rows.iter().find(|r| r.provider == provider.slug());
        let configured = provider.configured();
        let entitled = entitlement.permits(&provider);
        let status = if configured && !provider.requires_auth() {
            "no_auth_required"
        } else if configured && !provider.provisioned() {
            "not_provisioned"
        } else if configured {
            row.map_or("not_connected", |r| r.status.as_str())
        } else {
            "not_configured"
        };
        let mut actions = Vec::new();
        if configured && entitled && provider.requires_auth() && status != "not_provisioned" {
            actions.push(
                if status == "connected" {
                    "reconnect"
                } else {
                    "connect"
                }
                .into(),
            );
            if row.is_some_and(|r| r.auth_method.is_some()) || provider.is_salesforce() {
                actions.push("test".into());
            }
            if matches!(provider, Provider::Atlassian | Provider::Github) {
                actions.push("manual_token".into());
            }
        }
        if row.is_some_and(|r| r.auth_method.is_some()) {
            actions.push("disconnect".into());
        }
        connections.push(Connection {
            provider: provider.slug().into(),
            display_name: provider.display_name(),
            requires_auth: provider.requires_auth(),
            configured,
            entitled,
            status: status.into(),
            auth_method: row.and_then(|r| r.auth_method.clone()),
            account_id: row.and_then(|r| r.account_id.clone()),
            account_name: row.and_then(|r| r.account_name.clone()),
            resource_id: row.and_then(|r| r.resource_id.clone()),
            resource_name: row.and_then(|r| r.resource_name.clone()),
            error_code: row.and_then(|r| r.error_code.clone()),
            verified_at: row.and_then(|r| r.verified_at.map(|t| t.to_rfc3339())),
            actions,
        });
    }
    Ok(ConnectionSnapshot {
        version: 1,
        user_id: user.clone(),
        revision: rows.iter().map(|r| r.revision).max().unwrap_or(0),
        connections,
    })
}
