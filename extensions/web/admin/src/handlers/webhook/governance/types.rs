use std::borrow::Cow;
use std::fmt;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GovernanceDecision {
    Allow,
    Deny,
}

impl fmt::Display for GovernanceDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allow => f.write_str("allow"),
            Self::Deny => f.write_str("deny"),
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

#[derive(Debug, Serialize)]
pub struct EvaluatedRule {
    pub rule: &'static str,
    pub result: &'static str,
    pub detail: Cow<'static, str>,
}

pub(super) struct RuleEvaluation {
    pub decision: GovernanceDecision,
    pub reason: Cow<'static, str>,
    pub policy: Cow<'static, str>,
    pub rules: Vec<EvaluatedRule>,
}

pub(super) struct GovernanceContext<'a> {
    pub tool_name: &'a str,
    pub agent_scope: &'a str,
    pub session_id: &'a SessionId,
    pub user_id: &'a UserId,
    pub tool_input: Option<&'a serde_json::Value>,
}

pub(super) struct AuditRecord {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub tool_name: String,
    pub agent_id: Option<String>,
    pub agent_scope: String,
    pub decision: String,
    pub policy: String,
    pub reason: String,
    pub evaluated_rules: serde_json::Value, // JSON: protocol boundary
    pub plugin_id: Option<String>,
}

pub(super) struct AuthDenialParams<'a> {
    pub pool: &'a PgPool,
    pub session_id: &'a SessionId,
    pub tool_name: &'a str,
    pub agent_id: Option<&'a str>,
    pub plugin_id: Option<&'a str>,
}

pub(super) struct AuditParams<'a> {
    pub pool: &'a PgPool,
    pub user_id: &'a UserId,
    pub session_id: &'a SessionId,
    pub tool_name: &'a str,
    pub agent_id: Option<&'a str>,
    pub agent_scope: &'a str,
    pub evaluation: &'a RuleEvaluation,
    pub plugin_id: Option<&'a str>,
}
