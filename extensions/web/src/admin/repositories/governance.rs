use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::{GovernanceDecisionRow, GovernanceEvent};

pub async fn list_governance_decisions(
    pool: &Arc<PgPool>,
    search: Option<&str>,
) -> Result<Vec<GovernanceDecisionRow>, sqlx::Error> {
    let search_pattern = search
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));

    if let Some(ref pattern) = search_pattern {
        sqlx::query_as::<_, GovernanceDecisionRow>(
            "SELECT id, user_id, tool_name, agent_id, agent_scope, decision, policy, reason, created_at \
             FROM governance_decisions \
             WHERE user_id ILIKE $1 OR tool_name ILIKE $1 OR decision ILIKE $1 \
                OR reason ILIKE $1 OR policy ILIKE $1 OR agent_scope ILIKE $1 \
             ORDER BY created_at DESC \
             LIMIT 200",
        )
        .bind(pattern)
        .fetch_all(pool.as_ref())
        .await
    } else {
        sqlx::query_as::<_, GovernanceDecisionRow>(
            "SELECT id, user_id, tool_name, agent_id, agent_scope, decision, policy, reason, created_at \
             FROM governance_decisions \
             ORDER BY created_at DESC \
             LIMIT 200",
        )
        .fetch_all(pool.as_ref())
        .await
    }
}

pub async fn fetch_governance_events(
    pool: &Arc<PgPool>,
) -> Result<Vec<GovernanceEvent>, sqlx::Error> {
    sqlx::query_as::<_, GovernanceEvent>(
        r"SELECT id, user_id, tool_name, agent_id, decision, reason, created_at
        FROM governance_decisions
        ORDER BY
            CASE WHEN decision = 'deny' THEN 0 ELSE 1 END,
            created_at DESC
        LIMIT 50",
    )
    .fetch_all(pool.as_ref())
    .await
}
