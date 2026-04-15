use sqlx::PgPool;

use crate::types::{GovernanceDecisionRow, GovernanceEvent};

pub async fn list_governance_decisions(
    pool: &PgPool,
    search: Option<&str>,
) -> Result<Vec<GovernanceDecisionRow>, sqlx::Error> {
    let search_pattern = search.filter(|s| !s.is_empty()).map(|s| format!("%{s}%"));

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
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, GovernanceDecisionRow>(
            "SELECT id, user_id, tool_name, agent_id, agent_scope, decision, policy, reason, created_at \
             FROM governance_decisions \
             ORDER BY created_at DESC \
             LIMIT 200",
        )
        .fetch_all(pool)
        .await
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GovernanceCounts {
    pub total: i64,
    pub allowed: i64,
    pub denied: i64,
    pub secret_breaches: i64,
}

pub async fn fetch_governance_counts(pool: &PgPool) -> Result<GovernanceCounts, sqlx::Error> {
    let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        r"SELECT
            COUNT(*)::bigint,
            COUNT(*) FILTER (WHERE decision = 'allow')::bigint,
            COUNT(*) FILTER (WHERE decision = 'deny')::bigint,
            COUNT(*) FILTER (WHERE reason ILIKE '%secret%')::bigint
        FROM governance_decisions",
    )
    .fetch_one(pool)
    .await?;
    Ok(GovernanceCounts {
        total: row.0,
        allowed: row.1,
        denied: row.2,
        secret_breaches: row.3,
    })
}

pub async fn fetch_governance_events(pool: &PgPool) -> Result<Vec<GovernanceEvent>, sqlx::Error> {
    sqlx::query_as::<_, GovernanceEvent>(
        r"SELECT id, user_id, tool_name, agent_id, decision, reason, created_at
        FROM governance_decisions
        ORDER BY created_at DESC
        LIMIT 50",
    )
    .fetch_all(pool)
    .await
}
