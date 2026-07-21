//! Effective-permissions computation for the user-detail page.
//!
//! For a given user's roles, runs the pure
//! [`systemprompt_security::authz::resolve`] resolver against every gateway
//! route and every MCP server, returning per-entity Allow/Deny decisions
//! with the rule that decided. The view layer renders these as collapsible
//! sections under an "Effective Permissions" tab.
//!
//! Department rules are not represented here. `access_control_rules` stores
//! `rule_type = 'department'` and the access matrix and department rollups
//! both honour it, but the resolver's vocabulary
//! ([`systemprompt_security::authz::RuleType`]) is `User` and `Role` only, so
//! a grant that a department rule alone would confer is not reflected on this
//! tab. Closing that gap means widening the rule type in core, not
//! reimplementing precedence here.

use std::sync::Arc;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{McpServerId, RouteId, UserId};
use systemprompt_security::authz::{
    AccessControlRepository, AccessRule, Decision, EntityKind, EntityRef, ResolveInput, resolve,
};

use crate::error::AdminError;
use crate::handlers::shared;
use crate::repositories;
use crate::repositories::mcp::mcp_servers;

#[derive(Debug, Serialize, Clone)]
pub struct EntityDecision {
    // Why: polymorphic entity reference (gateway_route/mcp_server), no single typed-ID equivalent
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

pub async fn compute_effective_permissions(
    pool: &PgPool,
    user_id: &UserId,
    user_roles: &[String],
) -> EffectivePermissions {
    let gateway_ids = collect_gateway_ids().unwrap_or_default();
    let mcp_ids = collect_mcp_ids().unwrap_or_default();
    let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));

    let gateway_rules = repo
        .list_rules_bulk(EntityKind::GatewayRoute, &gateway_ids)
        .await
        .unwrap_or_default();
    let mcp_rules = repo
        .list_rules_bulk(EntityKind::McpServer, &mcp_ids)
        .await
        .unwrap_or_default();

    let mut gateway_routes = Vec::with_capacity(gateway_ids.len());
    for id in &gateway_ids {
        let rules = gateway_rules.get(id).cloned().unwrap_or_default();
        let default_included = repo
            .get_entity(EntityKind::GatewayRoute, id)
            .await
            .inspect_err(
                |e| tracing::warn!(error = %e, id = %id, "effective: gateway get_entity failed"),
            )
            .ok()
            .flatten()
            .map(|e| e.default_included);
        gateway_routes.push(decide(DecideArgs {
            entity_id: id,
            entity_type: "gateway_route",
            entity: EntityRef::GatewayRoute(RouteId::new(id.clone())),
            rules: &rules,
            user_id: user_id.as_str(),
            user_roles,
            default_included,
        }));
    }

    let mut mcp_servers = Vec::with_capacity(mcp_ids.len());
    for id in &mcp_ids {
        let rules = mcp_rules.get(id).cloned().unwrap_or_default();
        let default_included = repo
            .get_entity(EntityKind::McpServer, id)
            .await
            .inspect_err(
                |e| tracing::warn!(error = %e, id = %id, "effective: mcp get_entity failed"),
            )
            .ok()
            .flatten()
            .map(|e| e.default_included);
        mcp_servers.push(decide(DecideArgs {
            entity_id: id,
            entity_type: "mcp_server",
            entity: EntityRef::McpServer(McpServerId::new(id.clone())),
            rules: &rules,
            user_id: user_id.as_str(),
            user_roles,
            default_included,
        }));
    }

    EffectivePermissions {
        gateway_routes,
        mcp_servers,
    }
}

struct DecideArgs<'a> {
    entity_id: &'a str,
    entity_type: &'a str,
    entity: EntityRef,
    rules: &'a [AccessRule],
    user_id: &'a str,
    user_roles: &'a [String],
    default_included: Option<bool>,
}

fn decide(args: DecideArgs<'_>) -> EntityDecision {
    let DecideArgs {
        entity_id,
        entity_type,
        entity,
        rules,
        user_id,
        user_roles,
        default_included,
    } = args;
    let uid = UserId::new(user_id);
    let dec = resolve(ResolveInput {
        entity: &entity,
        rules,
        user_id: &uid,
        user_roles,
        default_included,
        parents: &[],
    });
    let (decision, reason) = match dec {
        Decision::Allow { .. } => (
            "allow".to_owned(),
            allow_reason(rules, &uid, user_roles, default_included.unwrap_or(false)),
        ),
        Decision::Deny { reason } => ("deny".to_owned(), reason.to_string()),
    };
    EntityDecision {
        entity_id: entity_id.to_owned(),
        decision,
        reason,
        matrix_url: format!(
            "/admin/access?tab={}#{}",
            if entity_type == "gateway_route" {
                "gateway"
            } else {
                "mcp"
            },
            entity_id
        ),
    }
}

/// Best-effort label for *why* a decision was Allow — the resolver only emits
/// reasons for Deny. Mirrors the resolver's specificity ordering.
fn allow_reason(
    rules: &[AccessRule],
    user_id: &UserId,
    user_roles: &[String],
    default_included: bool,
) -> String {
    use systemprompt_security::authz::{Access, RuleType};
    if rules.iter().any(|r| {
        r.rule_type == RuleType::User
            && r.rule_value.as_str() == user_id.as_str()
            && r.access == Access::Allow
    }) {
        return format!("user-level allow: {user_id}");
    }
    if let Some(rule) = rules.iter().find(|r| {
        r.rule_type == RuleType::Role
            && r.access == Access::Allow
            && user_roles.iter().any(|x| x == &r.rule_value)
    }) {
        return format!("role allow: {}", rule.rule_value);
    }
    if default_included {
        return "default included".to_owned();
    }
    "allow (resolver)".to_owned()
}

fn collect_gateway_ids() -> Result<Vec<String>, AdminError> {
    let profile_path = shared_path_or_err(shared::get_profile_path())?;
    let cfg = repositories::governance::gateway::get_gateway_config(&profile_path)
        .map_err(|e| AdminError::internal(e.to_string()))?;
    Ok(cfg.routes.into_iter().map(|r| r.id).collect())
}

fn collect_mcp_ids() -> Result<Vec<String>, AdminError> {
    let services_path = shared_path_or_err(shared::get_services_path())?;
    let servers = mcp_servers::list_mcp_servers(&services_path)
        .map_err(|e| AdminError::internal(e.to_string()))?;
    Ok(servers
        .into_iter()
        .map(|s| s.id.as_str().to_owned())
        .collect())
}

fn shared_path_or_err(
    r: Result<std::path::PathBuf, Box<axum::response::Response>>,
) -> Result<std::path::PathBuf, AdminError> {
    r.map_err(|resp| AdminError::internal(format!("path lookup failed: HTTP {}", resp.status())))
}
