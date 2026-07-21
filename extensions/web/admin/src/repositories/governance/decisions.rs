//! Raw decision-row queries: search, per-policy detail, and the recent feed.

use sqlx::PgPool;

use crate::types::GovernanceDecisionRow;

pub async fn list_decisions_for_policy(
    pool: &PgPool,
    policy_id: &str,
    limit: i64,
) -> Result<Vec<GovernanceDecisionRow>, sqlx::Error> {
    sqlx::query_as!(
        GovernanceDecisionRow,
        r#"SELECT id, user_id as "user_id!: _", tool_name,
                  agent_id as "agent_id: _", agent_scope,
                  decision, policy, reason, created_at
           FROM governance_decisions
           WHERE policy = $1
           ORDER BY created_at DESC
           LIMIT $2"#,
        policy_id,
        limit,
    )
    .fetch_all(pool)
    .await
}
