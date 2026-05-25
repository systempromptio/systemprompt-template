//! Wire and audit types for the `/api/public/hooks/govern` `PreToolUse` webhook.
//!
//! The on-the-wire response shape is dictated by the Anthropic Claude Code
//! hook contract ([`HookSpecificOutput`]). Internally everything is typed —
//! audit blobs serialize through [`DecisionAudit`] and per-policy trace
//! through [`ChainEntryOutcome`]; the previous `serde_json::json!` blobs are
//! gone.

use serde::Serialize;
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::identifiers::{PolicyId, SessionId, UserId};
use systemprompt_security::authz::{Decision, DecisionTag};
use systemprompt_security::policy::types::AccessScope;

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
            Decision::Allow { .. } => Self::Allow,
            Decision::Deny { .. } => Self::Deny,
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
pub struct GovernanceResponse {
    #[serde(rename = "hookSpecificOutput")]
    pub hook_specific_output: HookSpecificOutput,
}

#[derive(Debug, Clone, Serialize)]
pub struct HookSpecificOutput {
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

/// Per-policy outcome in the audit trace.
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "result", rename_all = "lowercase")]
pub enum ChainEntryResult {
    /// Policy was enabled and produced an allow.
    Pass,
    /// Policy was enabled and produced a deny.
    Fail,
    /// Policy was disabled in config, or skipped after a prior deny.
    Skip,
}

#[derive(Debug, Serialize, Clone)]
pub struct ChainEntryOutcome {
    pub policy_id: PolicyId,
    #[serde(flatten)]
    pub result: ChainEntryResult,
    pub detail: String,
}

/// Snapshot of the authenticated principal at evaluation time.
#[derive(Debug, Serialize, Clone)]
pub struct PrincipalSnapshot {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub agent_id: Option<String>,
    pub agent_scope: AccessScope,
}

/// What the chain was asked to evaluate.
#[derive(Debug, Serialize, Clone)]
pub struct AuditTarget {
    pub tool_name: String,
    pub plugin_id: Option<String>,
}

/// Typed audit blob serialized into `governance_decisions.evaluated_rules`.
///
/// Replaces the historical `serde_json::json!` payload. The Anthropic-style
/// `decision`/`reason` columns still get populated from the same data via the
/// repository layer.
#[derive(Debug, Serialize, Clone)]
pub struct DecisionAudit {
    pub decision: Decision,
    pub principal: PrincipalSnapshot,
    pub target: AuditTarget,
    pub chain: Vec<ChainEntryOutcome>,
}

pub(super) struct AuthDenialParams<'a> {
    pub pool: &'a Arc<PgPool>,
    pub session_id: &'a SessionId,
    pub tool_name: &'a str,
    pub agent_id: Option<&'a str>,
    pub plugin_id: Option<&'a str>,
}
