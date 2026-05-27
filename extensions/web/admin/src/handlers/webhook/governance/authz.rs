//! `POST /govern/authz` — extension webhook implementing
//! [`systemprompt_security::authz::AuthzDecisionHook`] as an HTTP endpoint.
//!
//! Core's gateway and MCP enforcement sites POST an [`AuthzRequest`] here;
//! this handler loads the matching rules from `access_control_rules`, runs
//! the pure deny-overrides resolver, audits the decision to
//! `governance_decisions`, and returns an [`AuthzDecision`] for core to act
//! on. The audit row's `policy` is `authz` regardless of `entity_type`, so
//! `infra logs audit` can correlate gateway and MCP decisions in one stream.

use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::identifiers::Actor;
use systemprompt_security::authz::{
    resolve, AccessControlRepository, AccessRule, AuthzDecision, AuthzRequest, Decision,
    DecisionTag, EntityRow, ResolveInput,
};

use crate::repositories::governance_grp::{insert_governance_decision, GovernanceDecisionRecord};

const POLICY_NAME: &str = "authz";

async fn load_rules(
    repo: &AccessControlRepository,
    req: &AuthzRequest,
) -> Result<(Vec<AccessRule>, Option<EntityRow>), Response> {
    let kind = req.entity.kind();
    let id = req.entity.id_str();
    let rules = repo
        .list_rules_for_entity(kind, id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, entity_type = %kind, entity_id = %id, "list_rules_for_entity failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "list_rules failed").into_response()
        })?;
    let entity = repo.get_entity(kind, id).await.map_err(|e| {
        tracing::error!(error = %e, entity_type = %kind, entity_id = %id, "get_entity failed");
        (StatusCode::INTERNAL_SERVER_ERROR, "get_entity failed").into_response()
    })?;
    Ok((rules, entity))
}

async fn audit_decision(
    pool: &PgPool,
    req: &AuthzRequest,
    rules: &[AccessRule],
    entity: Option<&EntityRow>,
    decision: &Decision,
) {
    let (decision_tag, reason_str, justification_opt): (DecisionTag, String, Option<String>) =
        match decision {
            Decision::Allow { .. } => (DecisionTag::Allow, String::new(), None),
            Decision::Deny { reason } => (DecisionTag::Deny, reason.to_string(), None),
        };
    let id = uuid::Uuid::new_v4().to_string();
    let entity_type_str = req.entity.kind().as_str();
    let entity_id_str = req.entity.id_str();
    let evaluated = serde_json::json!({
        "entity_type": entity_type_str,
        "entity_id": entity_id_str,
        "trace_id": req.trace_id.as_str(),
        "roles": req.roles,
        "attributes": req.attributes,
        "context": req.context,
        "entity": entity,
        "justification": justification_opt,
        "rules": rules,
    });
    let actor = Actor::user(req.user_id.clone());
    let record = GovernanceDecisionRecord {
        id: &id,
        actor: &actor,
        session_id: req.trace_id.as_str(),
        tool_name: entity_id_str,
        agent_id: None,
        // Why: authz decisions are entity-keyed, not agent-keyed; entity_type
        // remains in evaluated_rules above for forensic lookup. Writing it to
        // agent_scope was a historical column overload — removed.
        agent_scope: None,
        decision: decision_tag,
        policy: POLICY_NAME,
        reason: &reason_str,
        evaluated_rules: &evaluated,
        plugin_id: None,
        act_chain: &[],
    };
    if let Err(e) = insert_governance_decision(pool, &record).await {
        tracing::error!(error = %e, "Failed to record authz decision");
    }
}

pub async fn govern_authz(
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<AuthzRequest>,
) -> Response {
    let repo = AccessControlRepository::from_pool(Arc::clone(&pool));

    let (rules, entity) = match load_rules(&repo, &req).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let decision = resolve(ResolveInput {
        entity: &req.entity,
        rules: &rules,
        user_id: &req.user_id,
        user_roles: &req.roles,
        default_included: entity.as_ref().map(|e| e.default_included),
    });

    audit_decision(&pool, &req, &rules, entity.as_ref(), &decision).await;

    let resp = match decision {
        Decision::Allow { .. } => AuthzDecision::Allow,
        Decision::Deny { reason } => AuthzDecision::Deny {
            reason,
            policy: POLICY_NAME.to_string(),
        },
    };
    (StatusCode::OK, Json(resp)).into_response()
}
