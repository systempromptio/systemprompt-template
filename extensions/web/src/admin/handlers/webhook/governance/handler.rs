use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::models::auth::JwtAudience;

use crate::admin::types::{GovernQuery, HookEventPayload};

use super::audit;
use super::rules;
use super::scope;
use super::types::{AuditRecord, GovernanceContext, GovernanceResponse, HookSpecificOutput};
use crate::admin::handlers::webhook::helpers::{extract_bearer_token, get_jwt_config};

pub(crate) async fn govern_tool_use(
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Query(query): Query<GovernQuery>,
    Json(payload): Json<HookEventPayload>,
) -> Response {
    let Some(token) = extract_bearer_token(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Missing Authorization header"})), // JSON: protocol boundary
        )
            .into_response();
    };

    let (jwt_secret, jwt_issuer) = match get_jwt_config() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load JWT config");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal configuration error"})), // JSON: protocol boundary
            )
                .into_response();
        }
    };

    let claims = match systemprompt::oauth::validate_jwt_token(
        token,
        &jwt_secret,
        &jwt_issuer,
        &[
            JwtAudience::Resource("hook".to_string()),
            JwtAudience::Resource("plugin".to_string()),
        ],
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "Governance webhook JWT validation failed");
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid or expired token"})), // JSON: protocol boundary
            )
                .into_response();
        }
    };

    let user_id = &claims.sub;
    let tool_name = payload.tool_name.as_deref().unwrap_or("unknown");
    let agent_id = payload.agent_id.as_deref();
    let session_id = payload.session_id.as_deref().unwrap_or("unknown");
    let plugin_id = query.plugin_id.as_deref();

    let agent_scope = match agent_id {
        Some(id) => scope::resolve_agent_scope(id),
        None => "unknown".to_string(),
    };

    let ctx = GovernanceContext {
        tool_name,
        agent_scope: &agent_scope,
        session_id,
        user_id,
        tool_input: payload.tool_input.as_ref(),
    };

    let evaluation = rules::evaluate(&pool, &ctx).await;

    spawn_audit_recording(
        &pool,
        user_id,
        session_id,
        tool_name,
        agent_id,
        &agent_scope,
        &evaluation,
        plugin_id,
    );

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
        decision: evaluation.decision,
        reason: evaluation.reason,
        policy: evaluation.policy,
        agent_scope,
        tool_name: tool_name.to_string(),
        evaluated_rules: evaluation.rules,
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[allow(clippy::too_many_arguments)]
fn spawn_audit_recording(
    pool: &Arc<PgPool>,
    user_id: &str,
    session_id: &str,
    tool_name: &str,
    agent_id: Option<&str>,
    agent_scope: &str,
    evaluation: &super::types::RuleEvaluation,
    plugin_id: Option<&str>,
) {
    let p = pool.clone();
    let record = AuditRecord {
        user_id: user_id.to_string(),
        session_id: session_id.to_string(),
        tool_name: tool_name.to_string(),
        agent_id: agent_id.map(str::to_string),
        agent_scope: agent_scope.to_string(),
        decision: evaluation.decision.to_string(),
        policy: evaluation.policy.clone(),
        reason: evaluation.reason.clone(),
        evaluated_rules: serde_json::to_value(&evaluation.rules).unwrap_or_default(), // JSON: stored as JSONB
        plugin_id: plugin_id.map(str::to_string),
    };

    tokio::spawn(async move {
        audit::record_decision(&p, &record).await;
    });
}
