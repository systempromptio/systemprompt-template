//! Governance webhook entrypoint: authenticate, evaluate the policy chain, and
//! record an audit row before returning the `PreToolUse` decision.

mod authn;
mod governed;

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use sqlx::PgPool;
use systemprompt::identifiers::{CallId, SessionId};
use systemprompt::oauth::SessionCreationService;
use systemprompt::traits::SessionAnalytics;
use systemprompt_security::authz::Decision;
use systemprompt_security::policy::types::AccessScope;
use systemprompt_security::policy::{
    AgentScope, AuditOrigin, AuditTarget, ChainEntryOutcome, ChainEntryResult, DecisionAudit,
    PolicyContext, PrincipalSnapshot, record_decision,
};

use crate::types::webhook::{GovernQuery, HookEventPayload};

use super::engine::engine;
use super::scope;
use super::types::{AuthDenialParams, GovernanceDecision, GovernanceResponse, HookSpecificOutput};

use authn::{authenticate_request, deny_for_auth_failure};
use governed::{governed_input, governed_target};

fn header_str(headers: &HeaderMap, name: header::HeaderName) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned)
}

fn build_response(decision: &Decision, hook_event_name: &'static str) -> Response {
    // Why: lint-ok: http-error — builds the decision body itself
    let permission_decision = GovernanceDecision::from_decision(decision);
    let permission_decision_reason = match decision {
        Decision::Allow { .. } => None,
        Decision::Deny { reason } => Some(format!("[GOVERNANCE] {reason}")),
    };
    let response = GovernanceResponse {
        hook_specific_output: HookSpecificOutput {
            hook_event_name,
            permission_decision,
            permission_decision_reason,
        },
    };
    (StatusCode::OK, Json(response)).into_response()
}

pub(crate) async fn govern_tool_use(
    State(pool): State<Arc<PgPool>>,
    Extension(session_service): Extension<Arc<SessionCreationService>>,
    headers: HeaderMap,
    Query(query): Query<GovernQuery>,
    // JSON: protocol boundary — the third-party hook envelope, parsed into typed
    // events by `HookEventPayload::from_value` after the raw copy is retained
    Json(raw): Json<serde_json::Value>,
) -> Response {
    // Why: lint-ok: http-error — a hook answers 200 with a decision; an error
    // status reads as "hook unavailable" and lets the call through
    let (payload, _warnings) = HookEventPayload::from_value(raw);

    let target = governed_target(&payload);
    let input = governed_input(&payload);
    // Why: echo the caller's event back so a `UserPromptSubmit` gate is not handed
    // a `PreToolUse` envelope it would have to ignore.
    let response_event = if payload.prompt().is_some() {
        "UserPromptSubmit"
    } else {
        "PreToolUse"
    };
    let session_id = SessionId::new(payload.session_id());
    let agent_id = payload.common.agent_id.as_ref();
    let plugin_id = query.plugin_id.as_ref();

    let denial_params = AuthDenialParams {
        pool: &pool,
        session_id: &session_id,
        tool_name: target.as_str(),
        hook_event_name: response_event,
        agent_id,
        plugin_id,
        session_service: &session_service,
        headers: &headers,
    };

    let principal = match authenticate_request(&headers, &denial_params) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    let user_id = principal.user_id;

    let db_scope = scope::scope_from_user_roles(&pool, &user_id).await;
    let principal_scope = scope::higher_privilege(principal.token_scope, db_scope);
    let access_scope = agent_id.map_or(principal_scope, |id| {
        scope::higher_privilege(principal_scope, scope::resolve_agent_scope(id))
    });

    // Why: one POST is one call, and this hook is the only point that sees it —
    // an out-of-process agent has no second enforcement point to inherit from.
    let call_id = CallId::generate();
    let evaluation = engine().evaluate(&PolicyContext {
        target: target.clone(),
        agent_scope: AgentScope::User {
            user_id: user_id.clone(),
        },
        access_scope,
        session_id: &session_id,
        user_id: &user_id,
        input: &input,
        call_id: &call_id,
    });
    let (decision, chain) = (evaluation.decision, evaluation.chain);

    let audit = DecisionAudit {
        id: uuid::Uuid::new_v4().to_string(),
        call_id: call_id.as_str().to_owned(),
        origin: AuditOrigin::Governed,
        decision: decision.clone(),
        principal: PrincipalSnapshot {
            user_id,
            session_id: session_id.clone(),
            agent_session: None,
            agent_id: agent_id.cloned(),
            agent_scope: access_scope,
        },
        target: AuditTarget {
            tool_name: target.as_str().to_owned(),
            plugin_id: plugin_id.cloned(),
        },
        chain,
        approver: None,
        act_chain: Vec::new(),
        // Why: the tool-call webhook carries no conversational context; only
        // the gateway path knows one.
        context_id: None,
    };
    spawn_audit_recording(&pool, audit);

    build_response(&decision, response_event)
}

fn spawn_auth_denial(params: &AuthDenialParams<'_>, reason: &str) {
    let pool = Arc::<sqlx::Pool<sqlx::Postgres>>::clone(params.pool);
    let reason = reason.to_owned();
    let session_id = params.session_id.clone();
    let tool_name = params.tool_name.to_owned();
    let agent_id = params.agent_id.cloned();
    let plugin_id = params.plugin_id.cloned();
    let session_service = Arc::clone(params.session_service);
    let headers = params.headers.clone();

    tokio::spawn(async move {
        // Why: authentication failed before any real user was resolved. Every UserId
        // must be a real `users` row, so provision the anonymous principal for
        // this fingerprint (idempotent upsert) to carry the audit's foreign key.
        // Only user agent + locale are set because `compute_fingerprint` falls
        // back to exactly those two signals.
        let analytics = SessionAnalytics {
            user_agent: header_str(&headers, header::USER_AGENT),
            preferred_locale: header_str(&headers, header::ACCEPT_LANGUAGE),
            ..SessionAnalytics::default()
        };
        let user_id = match session_service.ensure_anonymous_user(&analytics).await {
            Ok((uid, _fingerprint)) => uid,
            Err(e) => {
                tracing::error!(
                    target: "governance.audit.write_failed",
                    error = %e,
                    session_id = %session_id,
                    "could not resolve anonymous principal; auth-denial audit dropped",
                );
                return;
            },
        };
        let audit = DecisionAudit {
            id: uuid::Uuid::new_v4().to_string(),
            // Why: refused before the chain ran, so no call identity was ever
            // minted for it — this denial is the whole of the call's history.
            call_id: CallId::generate().as_str().to_owned(),
            origin: AuditOrigin::Governed,
            decision: deny_for_auth_failure(&reason),
            principal: PrincipalSnapshot {
                user_id,
                session_id: session_id.clone(),
                agent_session: None,
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
                duration_ms: 0.0,
            }],
            approver: None,
            act_chain: Vec::new(),
            context_id: None,
        };
        if let Err(e) = record_decision(&pool, &audit).await {
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
        if let Err(e) = record_decision(&p, &audit).await {
            tracing::error!(
                target: "governance.audit.write_failed",
                error = %e,
                session_id = %session_id,
                "governance audit write failed; row dropped",
            );
        }
    });
}
