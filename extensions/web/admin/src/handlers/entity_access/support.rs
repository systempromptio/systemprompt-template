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
        // Why: 400-boundary classification; BadRequest carries the client-facing
        // message by design. lint-ok: error-adapt
        .map_err(|e| AdminError::BadRequest(format!("invalid entity_type: {e}")))
}

pub(super) fn repo(pool: &PgPool) -> AccessControlRepository {
    AccessControlRepository::from_pool(Arc::new(pool.clone()))
}

// Why: Resolve a rule subject to the `(rule_type, rule_value)` pair the
// access-control table stores.
//
// Why not `SubjectRef`: core's subject enum has no arm for an extension
// dimension, and `group` and `project` are exactly that — minted here with
// [`RuleType::extension`] and taught to the resolver by this crate's
// providers. Routing them through `SubjectRef` would mean widening a core
// type to carry a slug core never interprets.
//
// A group or project value is checked against its table: unlike a user id or
// a role name, a mistyped one would sit in the rules table matching nothing
// and looking granted.
pub(super) async fn parse_subject(
    pool: &PgPool,
    rule_type: &str,
    rule_value: &str,
) -> AdminResult<(RuleType, String)> {
    // Why: the rules body spells the field `rule_type` and the bulk template
    // spells it `subject_type`; one message names both so either caller is
    // pointed at their own field.
    let invalid = || {
        AdminError::BadRequest(
            "invalid rule_type (invalid subject_type): must be user, role, department, group or project"
                .to_owned(),
        )
    };
    match rule_type {
        // Why: `UserId` is a plain id with no validator, and on this instance
        // user ids are not UUIDs — invite- and SSO-provisioned accounts carry
        // slug/email-derived ids — so the boundary check is shape-light:
        // non-empty and colon-free (a colon would corrupt the rule key).
        "user" => {
            if rule_value.is_empty() || rule_value.contains(':') {
                return Err(invalid());
            }
            Ok((RuleType::USER, rule_value.to_owned()))
        },
        "role" => {
            if rule_value.trim().is_empty() {
                return Err(invalid());
            }
            Ok((RuleType::ROLE, rule_value.to_owned()))
        },
        "department" => {
            if repositories::departments::find_department_by_name(pool, rule_value)
                .await?
                .is_none()
            {
                return Err(AdminError::BadRequest(format!(
                    "No department {rule_value}"
                )));
            }
            Ok((RuleType::extension("department")?, rule_value.to_owned()))
        },
        "group" => {
            if repositories::groups::crud::find_group(pool, rule_value)
                .await?
                .is_none()
            {
                return Err(AdminError::BadRequest(format!("No group {rule_value}")));
            }
            Ok((RuleType::extension("group")?, rule_value.to_owned()))
        },
        "project" => {
            if repositories::projects::crud::find_project(pool, rule_value)
                .await?
                .is_none()
            {
                return Err(AdminError::BadRequest(format!("No project {rule_value}")));
            }
            Ok((RuleType::extension("project")?, rule_value.to_owned()))
        },
        _ => Err(invalid()),
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
        "gateway_route" => Ok(repositories::config::gateway::dispatchable_route_ids(
            systemprompt::loader::ServicesBootstrap::get()?,
        )),
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
