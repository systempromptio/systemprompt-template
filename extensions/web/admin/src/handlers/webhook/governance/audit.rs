use sqlx::PgPool;
use systemprompt::identifiers::Actor;

use crate::repositories::governance_grp::{insert_governance_decision, GovernanceDecisionRecord};

use super::types::AuditRecord;

pub(super) async fn record_decision(pool: &PgPool, record: &AuditRecord) {
    let id = uuid::Uuid::new_v4().to_string();
    let actor = Actor::from_tool_name(
        record.user_id.clone(),
        record.agent_id.as_deref(),
        &record.tool_name,
    );
    let dec_record = GovernanceDecisionRecord {
        id: &id,
        actor: &actor,
        session_id: record.session_id.as_str(),
        tool_name: &record.tool_name,
        agent_id: record.agent_id.as_deref(),
        agent_scope: &record.agent_scope,
        decision: &record.decision,
        policy: &record.policy,
        reason: &record.reason,
        evaluated_rules: &record.evaluated_rules,
        plugin_id: record.plugin_id.as_deref(),
        act_chain: &[],
    };

    if let Err(e) = insert_governance_decision(pool, &dec_record).await {
        tracing::error!(error = %e, "Failed to record governance decision");
    }
}
