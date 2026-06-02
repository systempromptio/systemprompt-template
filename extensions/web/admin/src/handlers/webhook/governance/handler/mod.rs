//! Governance webhook entrypoint: authenticate, evaluate the policy chain, and
//! record an audit row before returning the `PreToolUse` decision.

mod authn;
mod evaluate;

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use sqlx::PgPool;
use systemprompt::identifiers::SessionId;
use systemprompt::oauth::SessionCreationService;
use systemprompt_security::authz::Decision;
use systemprompt_security::policy::types::AccessScope;

use crate::types::webhook::GovernQuery;
use crate::types::webhook::HookEventPayload;

use super::audit;
use super::scope;
use super::types::{
    AuditTarget, AuthDenialParams, ChainEntryOutcome, ChainEntryResult, DecisionAudit,
    GovernanceDecision, GovernanceResponse, HookSpecificOutput, PrincipalSnapshot,
};

use authn::{authenticate_request, deny_for_auth_failure};
use evaluate::{evaluate, EvaluateInput};

fn build_response(decision: &Decision) -> Response {
    let permission_decision = GovernanceDecision::from_decision(decision);
    let permission_decision_reason = match decision {
        Decision::Allow { .. } => None,
        Decision::Deny { reason } => Some(format!("[GOVERNANCE] {reason}")),
    };
    let response = GovernanceResponse {
        hook_specific_output: HookSpecificOutput {
            hook_event_name: "PreToolUse",
            permission_decision,
            permission_decision_reason,
        },
    };
    (StatusCode::OK, Json(response)).into_response()
}

pub async fn govern_tool_use(
    State(pool): State<Arc<PgPool>>,
    Extension(session_service): Extension<Arc<SessionCreationService>>,
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
        session_service: &session_service,
        headers: &headers,
    };

    let principal = match authenticate_request(&headers, &denial_params) {
        Ok(p) => p,
        Err(resp) => return *resp,
    };
    let user_id = principal.user_id;

    let access_scope = match agent_id {
        Some(id) => scope::higher_privilege(principal.access_scope, scope::resolve_agent_scope(id)),
        None => principal.access_scope,
    };

    let (decision, chain) = evaluate(&EvaluateInput {
        tool_name,
        session_id: &session_id,
        user_id: &user_id,
        access_scope,
        tool_input: payload.tool_input(),
    });

    let audit = DecisionAudit {
        decision: decision.clone(),
        principal: PrincipalSnapshot {
            user_id,
            session_id: session_id.clone(),
            agent_id: agent_id.map(str::to_string),
            agent_scope: access_scope,
        },
        target: AuditTarget {
            tool_name: tool_name.to_string(),
            plugin_id: plugin_id.map(str::to_string),
        },
        chain,
    };
    spawn_audit_recording(&pool, audit);

    build_response(&decision)
}

fn spawn_auth_denial(params: &AuthDenialParams<'_>, reason: &str) {
    let pool = Arc::<sqlx::Pool<sqlx::Postgres>>::clone(params.pool);
    let reason = reason.to_string();
    let session_id = params.session_id.clone();
    let tool_name = params.tool_name.to_string();
    let agent_id = params.agent_id.map(str::to_string);
    let plugin_id = params.plugin_id.map(str::to_string);
    let session_service = Arc::clone(params.session_service);
    let headers = params.headers.clone();

    tokio::spawn(async move {
        // Authentication failed before any real user was resolved. Every UserId
        // must be a real `users` row, so provision the anonymous principal for
        // this fingerprint (idempotent upsert) to carry the audit's foreign key.
        let user_id = match session_service.ensure_anonymous_user(&headers, None).await {
            Ok((uid, _fingerprint)) => uid,
            Err(e) => {
                tracing::error!(
                    target: "governance.audit.write_failed",
                    error = %e,
                    session_id = %session_id,
                    "could not resolve anonymous principal; auth-denial audit dropped",
                );
                return;
            }
        };
        let audit = DecisionAudit {
            decision: deny_for_auth_failure(&reason),
            principal: PrincipalSnapshot {
                user_id,
                session_id: session_id.clone(),
                agent_id,
                agent_scope: AccessScope::Unknown,
            },
            target: AuditTarget {
                tool_name,
                plugin_id,
            },
            chain: vec![ChainEntryOutcome {
                policy_id: systemprompt::identifiers::PolicyId::new("authentication"),
                result: ChainEntryResult::Fail,
                detail: reason,
            }],
        };
        if let Err(e) = audit::record_decision(&pool, &audit).await {
            tracing::error!(
                target: "governance.audit.write_failed",
                error = %e,
                session_id = %session_id,
                "governance audit write failed; row dropped",
            );
        }
    });
}

fn spawn_audit_recording(pool: &Arc<PgPool>, audit: DecisionAudit) {
    let p = Arc::<sqlx::Pool<sqlx::Postgres>>::clone(pool);
    tokio::spawn(async move {
        let session_id = audit.principal.session_id.clone();
        if let Err(e) = audit::record_decision(&p, &audit).await {
            tracing::error!(
                target: "governance.audit.write_failed",
                error = %e,
                session_id = %session_id,
                "governance audit write failed; row dropped",
            );
        }
    });
}
