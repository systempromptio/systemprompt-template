use std::sync::Arc;

use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Serialize)]
pub(crate) struct GovernanceResponse {
    #[serde(rename = "hookSpecificOutput")]
    pub hook_specific_output: HookSpecificOutput,
}

#[derive(Debug, Serialize)]
pub(crate) struct HookSpecificOutput {
    #[serde(rename = "hookEventName")]
    pub hook_event_name: &'static str,
    #[serde(rename = "permissionDecision")]
    pub permission_decision: &'static str,
    #[serde(
        rename = "permissionDecisionReason",
        skip_serializing_if = "Option::is_none"
    )]
    pub permission_decision_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct EvaluatedRule {
    pub rule: &'static str,
    pub result: &'static str,
    pub detail: String,
}

pub(super) struct RuleEvaluation {
    pub decision: &'static str,
    pub reason: String,
    pub policy: String,
    pub rules: Vec<EvaluatedRule>,
}

pub(super) struct GovernanceContext<'a> {
    pub tool_name: &'a str,
    pub agent_scope: &'a str,
    pub session_id: &'a str,
    pub user_id: &'a str,
    pub tool_input: Option<&'a serde_json::Value>,
}

pub(super) struct AuditRecord {
    pub user_id: String,
    pub session_id: String,
    pub tool_name: String,
    pub agent_id: Option<String>,
    pub agent_scope: String,
    pub decision: String,
    pub policy: String,
    pub reason: String,
    pub evaluated_rules: serde_json::Value,
    pub plugin_id: Option<String>,
}

pub(super) struct AuthDenialParams<'a> {
    pub pool: &'a Arc<PgPool>,
    pub session_id: &'a str,
    pub tool_name: &'a str,
    pub agent_id: Option<&'a str>,
    pub plugin_id: Option<&'a str>,
}

pub(super) struct AuditParams<'a> {
    pub pool: &'a Arc<PgPool>,
    pub user_id: &'a str,
    pub session_id: &'a str,
    pub tool_name: &'a str,
    pub agent_id: Option<&'a str>,
    pub agent_scope: &'a str,
    pub evaluation: &'a RuleEvaluation,
    pub plugin_id: Option<&'a str>,
}
