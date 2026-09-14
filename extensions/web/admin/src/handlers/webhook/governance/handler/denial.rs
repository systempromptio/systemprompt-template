//! The audit row for a call refused before authentication resolved a user.

use std::sync::Arc;

use axum::http::{HeaderMap, header};
use systemprompt::identifiers::CallId;
use systemprompt::traits::SessionAnalytics;
use systemprompt_security::policy::{
    AuditOrigin, AuditTarget, ChainEntryOutcome, ChainEntryResult, DecisionAudit, record_decision,
};

use systemprompt_security::policy::types::AccessScope;

use super::super::types::AuthDenialParams;
use super::authn::deny_for_auth_failure;
use super::principal_snapshot;

fn header_str(headers: &HeaderMap, name: header::HeaderName) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned)
}

// Why: this path stays spawned, unlike the governed-decision audit above —
// it runs `ensure_anonymous_user` (an upsert plus fingerprint work) before
// the insert, and awaiting that on the response path of every unauthenticated
// probe would hand anonymous callers a write-amplification lever. The loss
// window is process shutdown only, the denial itself was still enforced, and
// failures are logged under `governance.audit.write_failed`.
pub(super) fn spawn_auth_denial(params: &AuthDenialParams<'_>, reason: &str) {
    let pool = Arc::<sqlx::Pool<sqlx::Postgres>>::clone(params.pool);
    let reason = reason.to_owned();
    let session_id = params.session_id.clone();
    let tool_name = params.tool_name.to_owned();
    let claimed = params.claimed.cloned();
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
            // Why: authentication is what failed, so nothing about this caller
            // was verified — no scope, no client, and the agent id it sent
            // stays a claim.
            principal: principal_snapshot(
                user_id,
                session_id.clone(),
                AccessScope::Unknown,
                None,
                claimed,
            ),
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
            trace_id: None,
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
