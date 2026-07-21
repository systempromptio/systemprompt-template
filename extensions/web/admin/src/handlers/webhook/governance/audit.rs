//! Persists governance decisions to the audit spine.

use sqlx::PgPool;
use systemprompt::identifiers::{Actor, AgentId, PluginId};
use systemprompt_security::authz::{Decision, DecisionTag};

use crate::repositories::governance::{GovernanceDecisionRecord, insert_governance_decision};

use super::types::DecisionAudit;

pub(super) async fn record_decision(
    pool: &PgPool,
    audit: &DecisionAudit,
) -> Result<(), sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let actor = Actor::from_tool_name(
        audit.principal.user_id.clone(),
        audit.principal.agent_id.as_ref().map(AgentId::as_str),
        &audit.target.tool_name,
    );
    let (decision_tag, reason_str, policy_str) = match &audit.decision {
        Decision::Allow { .. } => (
            DecisionTag::Allow,
            String::new(),
            "default_allow".to_owned(),
        ),
        Decision::Deny { reason } => {
            let policy_str = audit
                .chain
                .iter()
                .find(|e| matches!(e.result, super::types::ChainEntryResult::Fail))
                .map_or_else(|| "unknown".to_owned(), |e| e.policy_id.as_str().to_owned());
            (DecisionTag::Deny, reason.to_string(), policy_str)
        },
    };
    let evaluated_rules = serde_json::to_value(audit).unwrap_or(serde_json::Value::Null);

    let dec_record = GovernanceDecisionRecord {
        id: &id,
        actor: &actor,
        session_id: audit.principal.session_id.as_str(),
        tool_name: &audit.target.tool_name,
        agent_id: audit.principal.agent_id.as_ref().map(AgentId::as_str),
        agent_scope: Some(audit.principal.agent_scope),
        decision: decision_tag,
        policy: &policy_str,
        reason: &reason_str,
        evaluated_rules: &evaluated_rules,
        plugin_id: audit.target.plugin_id.as_ref().map(PluginId::as_str),
        act_chain: &[],
        context_id: None,
        task_id: None,
    };

    insert_governance_decision(pool, &dec_record).await
}
