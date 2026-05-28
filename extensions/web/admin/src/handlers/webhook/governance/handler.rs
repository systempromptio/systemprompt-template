use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::identifiers::{McpToolName, SessionId, UserId};
use systemprompt::models::auth::JwtAudience;
use systemprompt::oauth::OauthError;
use systemprompt_security::authz::{Decision, DenyReason, MatchedBy};
use systemprompt_security::policy::{types::AccessScope, AgentScope, McpToolInput, PolicyContext};

use crate::handlers::webhook::helpers::{extract_bearer_token, get_jwt_issuer};
use crate::types::webhook::GovernQuery;
use crate::types::webhook::HookEventPayload;

use super::audit;
use super::policy::{self, PolicyConfig};
use super::scope;
use super::types::{
    AuditTarget, AuthDenialParams, ChainEntryOutcome, ChainEntryResult, DecisionAudit,
    GovernanceDecision, GovernanceResponse, HookSpecificOutput, PrincipalSnapshot,
};

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

fn deny_for_auth_failure(reason: &str) -> Decision {
    Decision::Deny {
        reason: DenyReason::HookUnavailable {
            policy: format!("auth_failure: {reason}"),
        },
    }
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

    let agent_scope = agent_id.map_or(AccessScope::Unknown, scope::resolve_agent_scope);

    let (decision, chain) = evaluate(&EvaluateInput {
        tool_name,
        session_id: &session_id,
        user_id: &user_id,
        agent_scope,
        tool_input: payload.tool_input(),
    });

    let audit = DecisionAudit {
        decision: decision.clone(),
        principal: PrincipalSnapshot {
            user_id,
            session_id: session_id.clone(),
            agent_id: agent_id.map(str::to_string),
            agent_scope,
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

fn authenticate_request(
    headers: &HeaderMap,
    denial_params: &AuthDenialParams<'_>,
) -> Result<UserId, Box<Response>> {
    let Some(token) = extract_bearer_token(headers) else {
        let reason = "Missing Authorization header — tool call blocked";
        spawn_auth_denial(denial_params, reason);
        return Err(Box::new(build_response(&deny_for_auth_failure(reason))));
    };

    let jwt_issuer = match get_jwt_issuer() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load JWT config");
            let reason = "Internal configuration error — tool call blocked";
            spawn_auth_denial(denial_params, reason);
            return Err(Box::new(build_response(&deny_for_auth_failure(reason))));
        }
    };

    let expected_aud = "hook|plugin|api";
    let claims = systemprompt::oauth::validate_jwt_token(
        token,
        &jwt_issuer,
        &[
            JwtAudience::Resource("hook".to_string()),
            JwtAudience::Resource("plugin".to_string()),
            JwtAudience::Api,
        ],
    )
    .map_err(|e| {
        log_jwt_failure(&e, expected_aud, &jwt_issuer);
        let reason = "Invalid or expired token — tool call blocked";
        spawn_auth_denial(denial_params, reason);
        Box::new(build_response(&deny_for_auth_failure(reason)))
    })?;

    Ok(UserId::new(&claims.sub))
}

fn log_jwt_failure(err: &OauthError, expected_aud: &str, issuer: &str) {
    let (detail, message) = jwt_failure_detail(err);
    tracing::warn!(detail = %detail, expected_aud, issuer, "{}", message);
}

fn jwt_failure_detail(err: &OauthError) -> (String, &'static str) {
    match err {
        OauthError::TokenAlgMismatch { got, expected } => (
            format!("alg got={got} expected={expected}"),
            "Governance webhook JWT rejected: signing algorithm mismatch",
        ),
        OauthError::TokenMissingKid => (
            "missing kid header".to_string(),
            "Governance webhook JWT rejected: missing `kid` header",
        ),
        OauthError::TokenUnknownKid { kid } => (
            format!("unknown kid={kid}"),
            "Governance webhook JWT rejected: unknown signing key — token was minted under a \
             different RSA authority",
        ),
        OauthError::Expired(reason) => (
            format!("expired: {reason}"),
            "Governance webhook JWT rejected: token expired",
        ),
        other => (
            format!("{other}"),
            "Governance webhook JWT validation failed",
        ),
    }
}

struct EvaluateInput<'a> {
    tool_name: &'a str,
    session_id: &'a SessionId,
    user_id: &'a UserId,
    agent_scope: AccessScope,
    tool_input: Option<&'a serde_json::Value>,
}

/// Iterates the configured policy chain in order, calling each enabled
/// `GovernancePolicy::evaluate` and stopping on the first deny. Emits a
/// per-entry trace so the audit row preserves the full evaluation order, not
/// just the first-deny.
fn evaluate(input: &EvaluateInput<'_>) -> (Decision, Vec<ChainEntryOutcome>) {
    let tool_input = McpToolInput::new(
        input
            .tool_input
            .cloned()
            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
    );

    let ctx = PolicyContext {
        tool: McpToolName::new(input.tool_name),
        agent_scope: AgentScope::User {
            user_id: input.user_id.clone(),
        },
        access_scope: input.agent_scope,
        session_id: input.session_id,
        user_id: input.user_id,
        tool_input: &tool_input,
    };

    let mut chain_trace: Vec<ChainEntryOutcome> = Vec::new();
    let mut denied: Option<Decision> = None;

    let chain = policy::chain();
    for (cfg, policy) in chain.iter() {
        if !cfg.enabled {
            chain_trace.push(disabled_entry(cfg));
            continue;
        }
        if denied.is_some() {
            chain_trace.push(skipped_after_deny(cfg));
            continue;
        }
        let decision = policy.evaluate(&ctx);
        match &decision {
            Decision::Allow { matched_by } => {
                chain_trace.push(ChainEntryOutcome {
                    policy_id: policy.id(),
                    result: ChainEntryResult::Pass,
                    detail: allow_detail(matched_by),
                });
            }
            Decision::Deny { reason } => {
                chain_trace.push(ChainEntryOutcome {
                    policy_id: policy.id(),
                    result: ChainEntryResult::Fail,
                    detail: reason.to_string(),
                });
                denied = Some(decision);
            }
        }
    }
    drop(chain);

    let final_decision = denied.unwrap_or(Decision::Allow {
        matched_by: MatchedBy::DefaultIncluded,
    });
    (final_decision, chain_trace)
}

fn disabled_entry(cfg: &PolicyConfig) -> ChainEntryOutcome {
    ChainEntryOutcome {
        policy_id: systemprompt::identifiers::PolicyId::new(cfg.id.clone()),
        result: ChainEntryResult::Skip,
        detail: "Policy disabled in services/governance/config.yaml".to_string(),
    }
}

fn skipped_after_deny(cfg: &PolicyConfig) -> ChainEntryOutcome {
    ChainEntryOutcome {
        policy_id: systemprompt::identifiers::PolicyId::new(cfg.id.clone()),
        result: ChainEntryResult::Skip,
        detail: "Skipped — already denied by an earlier policy".to_string(),
    }
}

fn allow_detail(matched_by: &MatchedBy) -> String {
    match matched_by {
        MatchedBy::PolicyAllow { detail, .. } => detail.to_string(),
        MatchedBy::UserAllow => "user allow".to_string(),
        MatchedBy::RoleAllow { role } => format!("role allow: {role}"),
        MatchedBy::DefaultIncluded => "default included".to_string(),
    }
}

fn spawn_auth_denial(params: &AuthDenialParams<'_>, reason: &str) {
    let p = Arc::<sqlx::Pool<sqlx::Postgres>>::clone(params.pool);
    let audit = DecisionAudit {
        decision: deny_for_auth_failure(reason),
        principal: PrincipalSnapshot {
            user_id: UserId::new("anonymous"),
            session_id: params.session_id.clone(),
            agent_id: params.agent_id.map(str::to_string),
            // Why: authentication failed before any scope could be resolved.
            // Unknown is the documented "could-not-resolve" fallback variant.
            agent_scope: AccessScope::Unknown,
        },
        target: AuditTarget {
            tool_name: params.tool_name.to_string(),
            plugin_id: params.plugin_id.map(str::to_string),
        },
        chain: vec![ChainEntryOutcome {
            policy_id: systemprompt::identifiers::PolicyId::new("authentication"),
            result: ChainEntryResult::Fail,
            detail: reason.to_string(),
        }],
    };
    spawn_audit_recording(&p, audit);
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
