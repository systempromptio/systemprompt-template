//! Raw decision-row queries: search, per-policy detail, and the recent feed.

use sqlx::PgPool;

use crate::types::{GovernanceDecisionRow, GovernanceEvent};

pub async fn list_governance_decisions(
    pool: &PgPool,
    search: Option<&str>,
) -> Result<Vec<GovernanceDecisionRow>, sqlx::Error> {
    let pattern = search.filter(|s| !s.is_empty()).map(|s| format!("%{s}%"));

    if let Some(p) = pattern {
        sqlx::query_as!(
            GovernanceDecisionRow,
            r#"SELECT id, user_id as "user_id!: _", tool_name,
                      agent_id as "agent_id: _", agent_scope,
                      decision, policy, reason, created_at
               FROM governance_decisions
               WHERE user_id ILIKE $1 OR tool_name ILIKE $1 OR decision ILIKE $1
                  OR reason ILIKE $1 OR policy ILIKE $1 OR agent_scope ILIKE $1
               ORDER BY created_at DESC
               LIMIT 200"#,
            p,
        )
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as!(
            GovernanceDecisionRow,
            r#"SELECT id, user_id as "user_id!: _", tool_name,
                      agent_id as "agent_id: _", agent_scope,
                      decision, policy, reason, created_at
               FROM governance_decisions
               ORDER BY created_at DESC
               LIMIT 200"#,
        )
        .fetch_all(pool)
        .await
    }
}

/// Decisions filtered to a single policy id. Used by the per-policy detail
/// page so admins can see exactly which calls a given policy has fired on.
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

pub async fn fetch_governance_events(pool: &PgPool) -> Result<Vec<GovernanceEvent>, sqlx::Error> {
    sqlx::query_as!(
        GovernanceEvent,
        r#"SELECT id, user_id as "user_id!: _", tool_name,
                  agent_id as "agent_id: _", decision, reason, created_at
           FROM governance_decisions
           ORDER BY created_at DESC
           LIMIT 50"#,
    )
    .fetch_all(pool)
    .await
}
