//! One public connection model for browser accounts and every enrolled bridge.

use super::connector_oauth::Provider;
use crate::error::{AdminError, AdminResult};
use crate::repositories::users::{access_control, connector_accounts as repo, queries};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Connection {
    pub provider: String,
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

pub(crate) async fn entitled(
    pool: &PgPool,
    user: &UserId,
    provider: Provider,
) -> AdminResult<bool> {
    let identity = queries::find_identity_envelope(pool, user)
        .await?
        .ok_or_else(|| AdminError::Unauthorized("Account unavailable".into()))?;
    if identity.status != "active" {
        return Ok(false);
    }
    let subject = access_control::user_subject(pool, user, identity.roles).await;
    // Why: connector access follows the configured MCP server policy; a
    // tenant-specific marketplace or group must never grant access implicitly.
    let sections = vec![(
        "mcp_server".into(),
        "Connectors".into(),
        vec![(provider.slug().into(), provider.slug().into(), None)],
    )];
    let result = access_control::resolve_subject_matrix(pool, &subject, sections).await?;
    Ok(result.len() == 1
        && result
            .iter()
            .all(|s| s.rows.len() == 1 && s.rows[0].effective == "allow"))
}

pub(crate) async fn require_entitlement(
    pool: &PgPool,
    user: &UserId,
    provider: Provider,
) -> AdminResult<()> {
    if !entitled(pool, user, provider).await? {
        return Err(AdminError::Forbidden(
            "Connector MCP access required".into(),
        ));
    }
    Ok(())
}

pub(crate) async fn get_connections(
    pool: &PgPool,
    user: &UserId,
) -> AdminResult<ConnectionSnapshot> {
    let rows = repo::list_accounts(pool, user).await?;
    let mut connections = Vec::new();
    for provider in Provider::ALL {
        let row = rows.iter().find(|r| r.provider == provider.slug());
        let configured = provider.configured();
        let entitled = entitled(pool, user, provider).await?;
        let status = if configured {
            row.map_or("not_connected", |r| r.status.as_str())
        } else {
            "not_configured"
        };
        let mut actions = Vec::new();
        if configured && entitled {
            actions.push(
                if status == "connected" {
                    "reconnect"
                } else {
                    "connect"
                }
                .into(),
            );
            if row.is_some_and(|r| r.auth_method.is_some()) || provider == Provider::Salesforce {
                actions.push("test".into());
            }
            if provider != Provider::Salesforce {
                actions.push("manual_token".into());
            }
        }
        if row.is_some_and(|r| r.auth_method.is_some()) {
            actions.push("disconnect".into());
        }
        connections.push(Connection {
            provider: provider.slug().into(),
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
