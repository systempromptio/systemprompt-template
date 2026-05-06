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
use systemprompt_security::authz::{
    resolve, AccessControlRepository, AccessRule, AuthzDecision, AuthzRequest, Decision,
};

use crate::repositories::governance_grp::{insert_governance_decision, GovernanceDecisionRecord};

const POLICY_NAME: &str = "authz";

async fn load_rules(
    repo: &AccessControlRepository,
    req: &AuthzRequest,
) -> Result<(Vec<AccessRule>, bool), Response> {
    let rules = repo
        .list_rules_for_entity(req.entity_type, &req.entity_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, entity_type = %req.entity_type, entity_id = %req.entity_id, "list_rules_for_entity failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "list_rules failed").into_response()
        })?;
    let default_included = repo
        .get_default_included(req.entity_type, &req.entity_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, entity_type = %req.entity_type, entity_id = %req.entity_id, "get_default_included failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "get_default_included failed").into_response()
        })?;
    Ok((rules, default_included))
}

async fn audit_decision(
    pool: &PgPool,
    req: &AuthzRequest,
    rules: &[AccessRule],
    default_included: bool,
    decision: &Decision,
) {
    let (decision_str, reason_str, justification_opt) = match decision {
        Decision::Allow => ("allow", String::new(), None),
        Decision::Deny {
            reason,
            justification,
        } => ("deny", reason.clone(), justification.clone()),
    };
    let id = uuid::Uuid::new_v4().to_string();
    let entity_type_str = req.entity_type.as_str();
    let evaluated = serde_json::json!({
        "entity_type": entity_type_str,
        "entity_id": req.entity_id,
        "trace_id": req.trace_id.as_str(),
        "roles": req.roles,
        "department": req.department,
        "context": req.context,
        "default_included": default_included,
        "justification": justification_opt,
        "rules": rules,
    });
    let record = GovernanceDecisionRecord {
        id: &id,
        user_id: req.user_id.as_str(),
        session_id: req.trace_id.as_str(),
        tool_name: &req.entity_id,
        agent_id: None,
        agent_scope: entity_type_str,
        decision: decision_str,
        policy: POLICY_NAME,
        reason: &reason_str,
        evaluated_rules: &evaluated,
        plugin_id: None,
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

    let (rules, default_included) = match load_rules(&repo, &req).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let decision = resolve(
        &rules,
        req.user_id.as_str(),
        &req.roles,
        &req.department,
        default_included,
    );

    audit_decision(&pool, &req, &rules, default_included, &decision).await;

    let resp = match decision {
        Decision::Allow => AuthzDecision::Allow,
        Decision::Deny { reason, .. } => AuthzDecision::Deny {
            reason,
            policy: POLICY_NAME.to_string(),
        },
    };
    (StatusCode::OK, Json(resp)).into_response()
}
