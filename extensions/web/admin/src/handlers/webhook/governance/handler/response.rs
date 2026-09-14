//! Typed success and denial envelopes for synchronous governance hooks.
use super::super::types::{GovernanceDecision, GovernanceResponse, HookSpecificOutput};
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use systemprompt_security::authz::Decision;
pub(super) fn build_response(decision: &Decision, hook_event_name: &'static str) -> Response {
    // Why: lint-ok: http-error — builds the decision body itself
    let permission_decision = GovernanceDecision::from_decision(decision);
    let permission_decision_reason = match decision {
        Decision::Allow { .. } => None,
        // Why: the reason is deliberately not returned to the caller. Claude
        // Code renders `permissionDecisionReason` to the user, and a warning
        // rendered as a refusal reads as one; the finding's home is the audit
        // row and `infra logs governance report`.
        Decision::Warn { reason } => {
            tracing::warn!(%reason, "governance warn-mode finding; allowing the tool call");
            None
        },
        Decision::Deny { reason } => Some(format!("[GOVERNANCE] {reason}")),
        Decision::Pending { reason } => Some(format!("[GOVERNANCE] {reason}")),
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
