use sqlx::PgPool;

use super::types::AuditRecord;

pub(super) async fn record_decision(pool: &PgPool, record: &AuditRecord) {
    let id = uuid::Uuid::new_v4().to_string();

    let result = sqlx::query(
        "INSERT INTO governance_decisions \
         (id, user_id, session_id, tool_name, agent_id, agent_scope, \
          decision, policy, reason, evaluated_rules, plugin_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(&id)
    .bind(&record.user_id)
    .bind(&record.session_id)
    .bind(&record.tool_name)
    .bind(&record.agent_id)
    .bind(&record.agent_scope)
    .bind(&record.decision)
    .bind(&record.policy)
    .bind(&record.reason)
    .bind(&record.evaluated_rules)
    .bind(&record.plugin_id)
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::error!(error = %e, "Failed to record governance decision");
    }
}
