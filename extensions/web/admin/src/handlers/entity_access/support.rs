//! Validation, parsing, and entity-id collection helpers shared by the
//! entity-access handlers.
//!
//! The handlers in the parent module own the HTTP shape; this module owns the
//! small pure conversions (string -> typed enum) and the on-disk lookups that
//! feed the bulk/matrix endpoints.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt_security::authz::{Access, AccessControlRepository, EntityKind, RuleType};

use crate::error::{AdminError, AdminResult};
use crate::handlers::shared;
use crate::repositories;
use crate::repositories::mcp::mcp_servers;

pub(super) fn validate_entity_type(entity_type: &str) -> AdminResult<EntityKind> {
    use std::str::FromStr;
    EntityKind::from_str(entity_type)
        .map_err(|e| AdminError::BadRequest(format!("invalid entity_type: {e}")))
}

pub(super) fn repo(pool: &PgPool) -> AccessControlRepository {
    AccessControlRepository::from_pool(Arc::new(pool.clone()))
}

/// Rule types this screen can write.
///
/// Deliberately narrower than core's open vocabulary: the entity-access form
/// edits user and role grants, while department rules are owned by the
/// department screens. An unrecognised value is rejected rather than minted,
/// so a typo cannot create a dimension nothing resolves.
pub(super) fn parse_rule_type(s: &str) -> Option<RuleType> {
    match s {
        "user" => Some(RuleType::USER),
        "role" => Some(RuleType::ROLE),
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

pub(super) fn collect_entity_ids(entity_type: &str) -> AdminResult<Vec<String>> {
    match entity_type {
        "gateway_route" => {
            let profile_path = shared::get_profile_path()?;
            let cfg = repositories::config::gateway::get_gateway_config(&profile_path)
                .map_err(AdminError::internal)?;
            Ok(cfg.routes.into_iter().map(|r| r.id).collect())
        },
        "mcp_server" => {
            let services_path = shared::get_services_path()?;
            let servers =
                mcp_servers::list_mcp_servers(&services_path).map_err(AdminError::internal)?;
            Ok(servers
                .into_iter()
                .map(|s| s.id.as_str().to_owned())
                .collect())
        },
        _ => Ok(Vec::new()),
    }
}
