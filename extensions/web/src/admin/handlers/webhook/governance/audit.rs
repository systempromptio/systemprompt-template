use sqlx::PgPool;

use super::types::AuditRecord;

pub(super) async fn record_decision(pool: &PgPool, record: &AuditRecord) {
    let id = uuid::Uuid::new_v4().to_string();
    let user_id = record.user_id.as_str();
    let session_id = record.session_id.as_str();

    let result = sqlx::query!(
        "INSERT INTO governance_decisions \
         (id, user_id, session_id, tool_name, agent_id, agent_scope, \
          decision, policy, reason, evaluated_rules, plugin_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        id,
        user_id,
        session_id,
        record.tool_name,
        record.agent_id,
        record.agent_scope,
        record.decision,
        record.policy,
        record.reason,
        record.evaluated_rules,
        record.plugin_id,
    )
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::error!(error = %e, "Failed to record governance decision");
    }
}
