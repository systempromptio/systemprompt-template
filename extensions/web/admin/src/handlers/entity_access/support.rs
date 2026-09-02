//! Validation, parsing, and entity-id collection helpers shared by the
//! entity-access handlers.
//!
//! The handlers in the parent module own the HTTP shape; this module owns the
//! small pure conversions (string -> typed enum) and the on-disk lookups that
//! feed the bulk/matrix endpoints.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::loader::ServicesBootstrap;
use systemprompt::identifiers::{DepartmentId, RoleId, UserId};
use systemprompt_security::authz::{Access, AccessControlRepository, EntityKind, SubjectRef};

use crate::error::{AdminError, AdminResult};
use crate::handlers::shared;
use crate::repositories;
use crate::repositories::mcp::mcp_servers;

pub(super) fn validate_entity_type(entity_type: &str) -> AdminResult<EntityKind> {
    use std::str::FromStr;
    EntityKind::from_str(entity_type)
        // Why: 400-boundary classification; BadRequest carries the client-facing
        // message by design. lint-ok: error-adapt
        .map_err(|e| AdminError::BadRequest(format!("invalid entity_type: {e}")))
}

pub(super) fn repo(pool: &PgPool) -> AccessControlRepository {
    AccessControlRepository::from_pool(Arc::new(pool.clone()))
}

pub(super) fn parse_subject(rule_type: &str, rule_value: &str) -> Option<SubjectRef> {
    let subject = match rule_type {
        // Why: `UserId` is a plain id with no validator, but the value here is
        // untrusted request input and every user id in this system is a UUID.
        // Parsing it at the boundary rejects garbage without demanding a
        // validated `UserId` — which would be a ~500-call-site migration.
        "user" => {
            uuid::Uuid::parse_str(rule_value).ok()?;
            SubjectRef::User(UserId::new(rule_value))
        },
        "role" => SubjectRef::Role(RoleId::try_new(rule_value).ok()?),
        "department" => SubjectRef::Department(DepartmentId::try_new(rule_value).ok()?),
        _ => return None,
    };
    match subject {
        s @ (SubjectRef::User(_) | SubjectRef::Role(_)) => Some(s),
        // Why: department editing is owned by the department screens.
        SubjectRef::Department(_) => None,
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
            let services = ServicesBootstrap::get()
                // Why: lint-ok: error-adapt — ConfigLoadError is core's variant-less loader error.
                .map_err(|e| AdminError::internal(format!("services tree is not loaded: {e}")))?;
            Ok(repositories::config::gateway::dispatchable_route_ids(
                services,
            ))
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
