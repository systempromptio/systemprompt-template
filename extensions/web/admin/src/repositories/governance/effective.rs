//! Effective-permissions computation for the user-detail page.
//!
//! For a given user's roles, runs the pure
//! [`systemprompt_security::authz::resolver::resolve`] resolver against every
//! gateway route and every MCP server, returning per-entity Allow/Deny
//! decisions with the rule that decided. The view layer renders these as
//! collapsible sections under an "Effective Permissions" tab.
//!
//! Every subject dimension this extension declares participates, not just user
//! and role: the AD groups a user holds are looked up once per page via
//! [`crate::authz::subject_attributes_for`] and handed to the resolver with
//! the rest, so a grant a group rule alone confers shows up here exactly as it
//! does at the enforcement point.

use crate::error::AdminError;
use crate::handlers::shared;
use crate::repositories;
use crate::repositories::mcp::mcp_servers;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Serialize, Clone)]
pub struct EntityDecision {
    pub entity_id: String,
    pub decision: String,
    pub reason: String,
    pub matrix_url: String,
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct EffectivePermissions {
    pub gateway_routes: Vec<EntityDecision>,
    pub mcp_servers: Vec<EntityDecision>,
}

// Why: lint-ok: unused-pub — the internal fork still calls this.
pub async fn compute_effective_permissions(
    pool: &PgPool,
    user_id: &UserId,
    _user_roles: &[String],
) -> Result<EffectivePermissions, sqlx::Error> {
    let gateway_ids = collect_gateway_ids().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    let mcp_ids = collect_mcp_ids().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    let rows = |ids: Vec<String>| ids.into_iter().map(|id| (id.clone(), id, None)).collect();
    let matrix = repositories::users::access_control::resolve_user_matrix(
        pool,
        user_id,
        vec![
            ("gateway_route".into(), "Gateway".into(), rows(gateway_ids)),
            ("mcp_server".into(), "MCP".into(), rows(mcp_ids)),
        ],
    )
    .await?
    .ok_or(sqlx::Error::RowNotFound)?;
    let mut result = EffectivePermissions::default();
    for section in matrix.sections {
        let tab = if section.entity_type == "gateway_route" {
            "gateway"
        } else {
            "mcp"
        };
        let entries = section
            .rows
            .into_iter()
            .map(|row| EntityDecision {
                matrix_url: format!("/admin/access?tab={tab}#{}", row.entity_id),
                entity_id: row.entity_id,
                decision: row.effective,
                reason: row.source.detail,
            })
            .collect();
        if tab == "gateway" {
            result.gateway_routes = entries;
        } else {
            result.mcp_servers = entries;
        }
    }
    Ok(result)
}

fn collect_gateway_ids() -> Result<Vec<String>, AdminError> {
    let services = systemprompt::loader::ServicesBootstrap::get()
        .map_err(|e| AdminError::internal(e.to_string()))?;
    Ok(repositories::config::gateway::dispatchable_route_ids(
        services,
    ))
}

fn collect_mcp_ids() -> Result<Vec<String>, AdminError> {
    let services_path = shared::get_services_path()?;
    let servers = mcp_servers::list_mcp_servers(&services_path)
        .map_err(|e| AdminError::internal(e.to_string()))?;
    Ok(servers
        .into_iter()
        .map(|s| s.id.as_str().to_owned())
        .collect())
}
