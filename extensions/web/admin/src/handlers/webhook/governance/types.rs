//! Wire types for the `/api/public/hooks/govern` `PreToolUse` webhook.
//!
//! The on-the-wire response shape is dictated by the Anthropic Claude Code
//! hook contract ([`HookSpecificOutput`]). The audit blob types
//! (`DecisionAudit` and friends) live in [`systemprompt_security::policy`];
//! this module keeps only what is specific to this extension's HTTP surface.

use axum::http::HeaderMap;
use serde::Serialize;
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::identifiers::{PluginId, SessionId};
use systemprompt::oauth::SessionCreationService;
use systemprompt_security::authz::{Decision, DecisionTag};
use systemprompt_security::policy::ClaimedAgent;

/// Anthropic-mandated wire enum for `permissionDecision`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GovernanceDecision {
    Allow,
    Deny,
}

impl GovernanceDecision {
    pub const fn from_decision(d: &Decision) -> Self {
        match d {
            // Why: warn is an allow to the caller. The policy that warned is
            // already on the `governance_decisions` row, so answering the hook
            // with a deny reason here would block the call the mode exists to
            // let through.
            Decision::Allow { .. } | Decision::Warn { .. } => Self::Allow,
            // Why: the Anthropic hook contract has no third answer, and this
            // webhook answers synchronously — it cannot park the call and
            // resume it. A hold degrades to a deny rather than an allow.
            Decision::Deny { .. } | Decision::Pending { .. } => Self::Deny,
        }
    }
}

impl From<GovernanceDecision> for DecisionTag {
    fn from(d: GovernanceDecision) -> Self {
        match d {
            GovernanceDecision::Allow => Self::Allow,
            GovernanceDecision::Deny => Self::Deny,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub(super) struct GovernanceResponse {
    #[serde(rename = "hookSpecificOutput")]
    pub hook_specific_output: HookSpecificOutput,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct HookSpecificOutput {
    #[serde(rename = "hookEventName")]
    pub hook_event_name: &'static str,
    #[serde(rename = "permissionDecision")]
    pub permission_decision: GovernanceDecision,
    #[serde(
        rename = "permissionDecisionReason",
        skip_serializing_if = "Option::is_none"
    )]
    pub permission_decision_reason: Option<String>,
}

pub(super) struct AuthDenialParams<'a> {
    pub pool: &'a Arc<PgPool>,
    pub session_id: &'a SessionId,
    pub tool_name: &'a str,
    // Why: Echoed into the response envelope so a `UserPromptSubmit` caller is not
    // answered with a `PreToolUse` denial it has to reinterpret.
    pub hook_event_name: &'static str,
    pub claimed: Option<&'a ClaimedAgent>,
    pub plugin_id: Option<&'a PluginId>,
    pub session_service: &'a Arc<SessionCreationService>,
    pub headers: &'a HeaderMap,
}
