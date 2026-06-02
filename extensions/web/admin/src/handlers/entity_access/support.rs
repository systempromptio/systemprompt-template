//! Validation, parsing, and entity-id collection helpers shared by the
//! entity-access handlers.
//!
//! The handlers in the parent module own the HTTP shape; this module owns the
//! small pure conversions (string -> typed enum) and the on-disk lookups that
//! feed the bulk/matrix endpoints.

use std::sync::Arc;

use axum::{http::StatusCode, response::Response};
use sqlx::PgPool;
use systemprompt_security::authz::{Access, AccessControlRepository, EntityKind, RuleType};

use crate::handlers::shared;
use crate::repositories::{self, mcp_servers};

pub(super) fn validate_entity_type(entity_type: &str) -> Result<EntityKind, Box<Response>> {
    use std::str::FromStr;
    EntityKind::from_str(entity_type).map_err(|_| {
        Box::new(shared::error_response(
            StatusCode::BAD_REQUEST,
            "invalid entity_type",
        ))
    })
}

pub(super) fn repo(pool: &PgPool) -> AccessControlRepository {
    AccessControlRepository::from_pool(Arc::new(pool.clone()))
}

pub(super) fn parse_rule_type(s: &str) -> Option<RuleType> {
    match s {
        "user" => Some(RuleType::User),
        "role" => Some(RuleType::Role),
        _ => None,
    }
}

pub(super) fn parse_access(s: &str) -> Option<Access> {
    match s {
        "allow" => Some(Access::Allow),
        "deny" => Some(Access::Deny),
        _ => None,
    }
}

pub(super) fn collect_entity_ids(entity_type: &str) -> Result<Vec<String>, Box<Response>> {
    match entity_type {
        "gateway_route" => {
            let profile_path = shared::get_profile_path()?;
            let cfg = repositories::get_gateway_config(&profile_path).map_err(|e| {
                tracing::error!(error = %e, "Failed to load gateway config");
                Box::new(shared::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to load gateway",
                ))
            })?;
            Ok(cfg.routes.into_iter().map(|r| r.id).collect())
        }
        "mcp_server" => {
            let services_path = shared::get_services_path()?;
            let servers = mcp_servers::list_mcp_servers(&services_path).map_err(|e| {
                tracing::error!(error = %e, "Failed to load MCP servers");
                Box::new(shared::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to load MCP servers",
                ))
            })?;
            Ok(servers
                .into_iter()
                .map(|s| s.id.as_str().to_string())
                .collect())
        }
        _ => Ok(Vec::new()),
    }
}
