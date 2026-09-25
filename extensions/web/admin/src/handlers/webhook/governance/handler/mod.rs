//! Governance webhook entrypoint: authenticate, evaluate the policy chain, and
//! record an audit row before returning the `PreToolUse` decision.

mod authn;
mod denial;
mod governed;
mod response;
use response::build_response;

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use sqlx::PgPool;
use systemprompt::identifiers::{CallId, ClientId, PluginId, SessionId, UserId};
use systemprompt::oauth::SessionCreationService;
use systemprompt_security::authz::Decision;
use systemprompt_security::policy::types::AccessScope;
use systemprompt_security::policy::{
    AgentScope, AuditOrigin, AuditTarget, DecisionAudit, GovernedInput, GovernedTarget,
    PolicyContext, PrincipalSnapshot, record_decision,
};

use crate::repositories::dashboard::usage_aggregations::ingestion;
use crate::types::webhook::{GovernQuery, HookEventPayload};

use super::engine::engine;
use super::types::AuthDenialParams;

use authn::{authenticate_request, deny_for_auth_failure};
use denial::spawn_auth_denial;
use governed::{governed_input, governed_target};


// Why: the hook payload's agent id is asserted by the caller, not verified
// from the credential, so it lands in `claimed`. Core writes `agent_id` to the
// verified identity column and keeps `claimed` in the audit blob only, where
// it is never an input to a decision.
pub(super) fn principal_snapshot(
    user_id: UserId,
    session_id: SessionId,
    agent_scope: AccessScope,
    client_id: Option<ClientId>,
    _claimed: Option<String>,
) -> PrincipalSnapshot {
    PrincipalSnapshot {
        user_id,
        session_id,
        agent_session: None,
        agent_id: None,
        agent_scope,
        client_id,
    }
}

fn response_event(payload: &HookEventPayload) -> &'static str {
    if payload.prompt().is_some() {
        "UserPromptSubmit"
    } else {
        "PreToolUse"
    }
}

// Why: the access scope must resolve to a live account before the chain
// runs; an unavailable answer is a denial, never a default.
async fn resolve_access_scope(
    pool: &PgPool,
    user_id: &UserId,
    denial: &AuthDenialParams<'_>,
) -> Result<AccessScope, &'static str> {
    match crate::authz::account_scope(pool, user_id).await {
        Ok(AccessScope::Admin) => Ok(AccessScope::Admin),
        Ok(AccessScope::User) => Ok(AccessScope::User),
        result => {
            tracing::warn!(?result, %user_id, "authorization_unavailable: active account");
            spawn_auth_denial(denial, "Active account authorization unavailable");
            Err("Active account authorization unavailable")
        },
    }
}

struct Governed<'a> {
    target: &'a GovernedTarget,
    input: &'a GovernedInput,
    session_id: &'a SessionId,
    user_id: UserId,
    client_id: Option<ClientId>,
    access_scope: AccessScope,
    _claimed: Option<String>,
    plugin_id: Option<&'a PluginId>,
}

// Why: one POST is one call, and this hook is the only point that sees it —
// an out-of-process agent has no second enforcement point to inherit from.
async fn evaluate_and_record(
    pool: &PgPool,
    governed: Governed<'_>,
    denial: &AuthDenialParams<'_>,
) -> Result<Decision, &'static str> {
    let call_id = CallId::generate();
    let engine = engine().map_err(|error| {
        tracing::error!(error = %error, "governance engine unavailable");
        spawn_auth_denial(denial, "Governance engine unavailable");
        "Governance engine unavailable"
    })?;
    let evaluation = engine.evaluate(&PolicyContext {
        target: governed.target.clone(),
        agent_scope: AgentScope::User {
            user_id: governed.user_id.clone(),
        },
        access_scope: governed.access_scope,
        session_id: governed.session_id,
        user_id: &governed.user_id,
        input: governed.input,
        call_id: &call_id,
    });
    let audit = DecisionAudit {
        id: uuid::Uuid::new_v4().to_string(),
        call_id,
        origin: AuditOrigin::Governed,
        decision: evaluation.decision.clone(),
        principal: principal_snapshot(
            governed.user_id,
            governed.session_id.clone(),
            governed.access_scope,
            governed.client_id,
            None,
        ),
        target: AuditTarget {
            tool_name: governed.target.as_str().to_owned(),
            plugin_id: governed.plugin_id.cloned(),
            tool_use_id: None,
        },
        chain: evaluation.chain,
        approver: None,
        act_chain: Vec::new(),
        // Why: the tool-call webhook carries no conversational context; only
        // the gateway path knows one.
        context_id: Some(systemprompt::identifiers::ContextId::derived_from_session(
            governed.session_id,
        )),
        trace_id: Some(systemprompt::identifiers::TraceId::generate().to_string()),
    };
    record_decision(pool, &audit).await.map_err(|error| {
        tracing::error!(%error, "Governance audit unavailable; denying execution");
        "Governance audit unavailable"
    })?;
    Ok(evaluation.decision)
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
    let response_event = response_event(&payload);
    let session_id = SessionId::new(payload.session_id());
    // Why: the hook body's agent id is a self-report — a Claude Code subagent
    // id, never a platform agent. It is kept for display and never becomes an
    // identity or a scope input.
    let claimed = payload.common.agent_id.as_ref().map(ToString::to_string);
    let plugin_id = query.plugin_id.as_ref();
    let denial_params = AuthDenialParams {
        pool: &pool,
        session_id: &session_id,
        tool_name: target.as_str(),
        hook_event_name: response_event,
        plugin_id,
        session_service: &session_service,
        headers: &headers,
    };

    let principal = match authenticate_request(&headers, &denial_params) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    if let Err(reason) = payload.validate_ingestion() {
        return build_response(&deny_for_auth_failure(&reason), response_event);
    }
    if let Err(error) =
        ingestion::assert_ingestion_owner(&pool, &session_id, &principal.user_id).await
    {
        tracing::error!(%error, "Hook identity binding failed");
        return build_response(
            &deny_for_auth_failure("Session binding unavailable"),
            response_event,
        );
    }
    let access_scope = match resolve_access_scope(&pool, &principal.user_id, &denial_params).await {
        Ok(scope) => scope,
        Err(reason) => return build_response(&deny_for_auth_failure(reason), response_event),
    };

    let governed = Governed {
        target: &target,
        input: &input,
        session_id: &session_id,
        user_id: principal.user_id,
        client_id: principal.client_id,
        access_scope,
        _claimed: claimed,
        plugin_id,
    };
    match evaluate_and_record(&pool, governed, &denial_params).await {
        Ok(decision) => build_response(&decision, response_event),
        Err(reason) => build_response(&deny_for_auth_failure(reason), response_event),
    }
}
