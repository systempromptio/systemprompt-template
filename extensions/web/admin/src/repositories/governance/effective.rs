//! Effective-permissions computation for the user-detail page.
//!
//! For a given user's roles, runs the pure
//! [`systemprompt_security::authz::resolve`] resolver against every gateway
//! route and every MCP server, returning per-entity Allow/Deny decisions
//! with the rule that decided. The view layer renders these as collapsible
//! sections under an "Effective Permissions" tab.
//!
//! Every subject dimension this extension declares participates, not just user
//! and role: the department a user belongs to is looked up once per page via
//! [`crate::authz::subject_attributes_for`] and handed to the resolver with
//! the rest, so a grant a department rule alone confers shows up here exactly
//! as it does at the enforcement point.

use std::sync::Arc;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{McpServerId, RouteId, UserId};
use systemprompt_security::authz::{
    AccessControlRepository, AccessRule, Decision, EntityKind, EntityRef, MatchedBy, ResolveInput,
    SubjectAttributes, SubjectDimension, resolve,
};

use crate::authz::{dimensions, subject_attributes_for};
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
    let attributes = subject_attributes_for(pool, user_id).await;
    let dimensions = dimensions(pool);

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
            attributes: &attributes,
            dimensions,
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
            attributes: &attributes,
            dimensions,
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
    attributes: &'a SubjectAttributes,
    dimensions: &'a [SubjectDimension],
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
        attributes,
        dimensions,
    } = args;
    let uid = UserId::new(user_id);
    let dec = resolve(ResolveInput {
        entity: &entity,
        rules,
        user_id: &uid,
        user_roles,
        default_included,
        parents: &[],
        attributes,
        dimensions,
    });
    let (decision, reason) = match dec {
        Decision::Allow { matched_by } => ("allow".to_owned(), allow_reason(&uid, &matched_by)),
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

/// Label for *why* a decision was Allow.
///
/// The resolver only spells out its reasoning for Deny, but it does say which
/// band matched, so this reads the answer off `MatchedBy` rather than
/// re-deriving it. The previous version re-walked the rules in its own
/// precedence order, which is the kind of second implementation that drifts.
fn allow_reason(user_id: &UserId, matched_by: &MatchedBy) -> String {
    match matched_by {
        MatchedBy::UserAllow => format!("user-level allow: {user_id}"),
        MatchedBy::RoleAllow { role } => format!("role allow: {role}"),
        MatchedBy::AttributeAllow { rule_type, value } => format!("{rule_type} allow: {value}"),
        MatchedBy::DefaultIncluded => "default included".to_owned(),
        MatchedBy::PolicyAllow { policy_id, detail } => format!("policy {policy_id}: {detail}"),
    }
}

fn collect_gateway_ids() -> Result<Vec<String>, AdminError> {
    let profile_path = shared::get_profile_path()?;
    let cfg = repositories::config::gateway::get_gateway_config(&profile_path)
        .map_err(|e| AdminError::internal(e.to_string()))?;
    Ok(cfg.routes.into_iter().map(|r| r.id).collect())
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
