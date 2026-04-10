use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt::models::auth::JwtAudience;

use crate::types::webhook::GovernQuery;
use crate::types::webhook::HookEventPayload;

use super::audit;
use super::rules;
use super::scope;
use super::types::{
    AuditParams, AuditRecord, AuthDenialParams, GovernanceContext, GovernanceResponse,
    HookSpecificOutput, RuleEvaluation,
};
use crate::handlers::webhook::helpers::{extract_bearer_token, get_jwt_config};

fn build_deny_response(reason: &str) -> Response {
    let response = GovernanceResponse {
        hook_specific_output: HookSpecificOutput {
            hook_event_name: "PreToolUse",
            permission_decision: "deny",
            permission_decision_reason: Some(format!("[GOVERNANCE] {reason}")),
        },
    };

    (StatusCode::OK, Json(response)).into_response()
}

pub async fn govern_tool_use(
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Query(query): Query<GovernQuery>,
    Json(raw): Json<serde_json::Value>,
) -> Response {
    let (payload, _warnings) = HookEventPayload::from_value(raw);

    let tool_name = payload.tool_name().unwrap_or("unknown");
    let session_id = SessionId::new(payload.session_id());
    let agent_id = payload.common.agent_id.as_deref();
    let plugin_id = query.plugin_id.as_deref();

    let denial_params = AuthDenialParams {
        pool: &pool,
        session_id: &session_id,
        tool_name,
        agent_id,
        plugin_id,
    };

    let user_id = match authenticate_request(&headers, &denial_params) {
        Ok(uid) => uid,
        Err(resp) => return *resp,
    };

    let agent_scope = agent_id.map_or_else(|| "unknown".to_string(), scope::resolve_agent_scope);

    let evaluation = evaluate_and_audit(&EvaluateInput {
        pool: &pool,
        payload: &payload,
        tool_name,
        session_id: &session_id,
        user_id: &user_id,
        agent_id,
        agent_scope: &agent_scope,
        plugin_id,
    });

    build_evaluation_response(&evaluation)
}

fn authenticate_request(
    headers: &HeaderMap,
    denial_params: &AuthDenialParams<'_>,
) -> Result<UserId, Box<Response>> {
    let Some(token) = extract_bearer_token(headers) else {
        let reason = "Missing Authorization header — tool call blocked";
        spawn_auth_denial(denial_params, reason);
        return Err(Box::new(build_deny_response(reason)));
    };

    let (jwt_secret, jwt_issuer) = match get_jwt_config() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load JWT config");
            let reason = "Internal configuration error — tool call blocked";
            spawn_auth_denial(denial_params, reason);
            return Err(Box::new(build_deny_response(reason)));
        }
    };

    let claims = systemprompt::oauth::validate_jwt_token(
        token,
        &jwt_secret,
        &jwt_issuer,
        &[
            JwtAudience::Resource("hook".to_string()),
            JwtAudience::Resource("plugin".to_string()),
            JwtAudience::Api,
        ],
    )
    .map_err(|e| {
        tracing::warn!(error = %e, "Governance webhook JWT validation failed");
        let reason = "Invalid or expired token — tool call blocked";
        spawn_auth_denial(denial_params, reason);
        Box::new(build_deny_response(reason))
    })?;

    Ok(UserId::new(&claims.sub))
}

struct EvaluateInput<'a> {
    pool: &'a Arc<PgPool>,
    payload: &'a HookEventPayload,
    tool_name: &'a str,
    session_id: &'a SessionId,
    user_id: &'a UserId,
    agent_id: Option<&'a str>,
    agent_scope: &'a str,
    plugin_id: Option<&'a str>,
}

fn evaluate_and_audit(input: &EvaluateInput<'_>) -> RuleEvaluation {
    let ctx = GovernanceContext {
        tool_name: input.tool_name,
        agent_scope: input.agent_scope,
        session_id: input.session_id,
        user_id: input.user_id,
        tool_input: input.payload.tool_input(),
    };

    let evaluation = rules::evaluate(&ctx);

    spawn_audit_recording(&AuditParams {
        pool: input.pool,
        user_id: input.user_id,
        session_id: input.session_id,
        tool_name: input.tool_name,
        agent_id: input.agent_id,
        agent_scope: input.agent_scope,
        evaluation: &evaluation,
        plugin_id: input.plugin_id,
    });

    evaluation
}

fn build_evaluation_response(evaluation: &RuleEvaluation) -> Response {
    let deny_reason = if evaluation.decision == "deny" {
        Some(format!("[GOVERNANCE] {}", evaluation.reason))
    } else {
        None
    };

    let response = GovernanceResponse {
        hook_specific_output: HookSpecificOutput {
            hook_event_name: "PreToolUse",
            permission_decision: evaluation.decision,
            permission_decision_reason: deny_reason,
        },
    };

    (StatusCode::OK, Json(response)).into_response()
}

fn spawn_auth_denial(params: &AuthDenialParams<'_>, reason: &str) {
    let p = params.pool.clone();
    let record = AuditRecord {
        user_id: UserId::anonymous(),
        session_id: params.session_id.clone(),
        tool_name: params.tool_name.to_string(),
        agent_id: params.agent_id.map(str::to_string),
        agent_scope: "unauthenticated".to_string(),
        decision: "deny".to_string(),
        policy: "auth_failure".to_string(),
        reason: reason.to_string(),
        evaluated_rules: serde_json::json!([{"rule": "authentication", "result": "fail", "detail": reason}]),
        plugin_id: params.plugin_id.map(str::to_string),
    };

    tokio::spawn(async move {
        audit::record_decision(&p, &record).await;
    });
}

fn spawn_audit_recording(params: &AuditParams<'_>) {
    let p = params.pool.clone();
    let record = AuditRecord {
        user_id: params.user_id.clone(),
        session_id: params.session_id.clone(),
        tool_name: params.tool_name.to_string(),
        agent_id: params.agent_id.map(str::to_string),
        agent_scope: params.agent_scope.to_string(),
        decision: params.evaluation.decision.to_string(),
        policy: params.evaluation.policy.clone(),
        reason: params.evaluation.reason.clone(),
        evaluated_rules: serde_json::to_value(&params.evaluation.rules).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to serialize governance evaluation rules");
            serde_json::Value::Null
        }),
        plugin_id: params.plugin_id.map(str::to_string),
    };

    tokio::spawn(async move {
        audit::record_decision(&p, &record).await;
    });
}
