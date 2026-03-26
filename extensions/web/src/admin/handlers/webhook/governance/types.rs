use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct GovernanceResponse {
    pub decision: &'static str,
    pub reason: String,
    pub policy: String,
    pub agent_scope: String,
    pub tool_name: String,
    pub evaluated_rules: Vec<EvaluatedRule>,
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
    pub tool_input: Option<&'a serde_json::Value>, // JSON: required by HookEventPayload contract
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
    pub evaluated_rules: serde_json::Value, // JSON: protocol boundary — stored as JSONB
    pub plugin_id: Option<String>,
}
