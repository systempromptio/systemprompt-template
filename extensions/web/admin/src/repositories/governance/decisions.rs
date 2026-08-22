//! Raw decision-row queries: search, per-policy detail, and the recent feed.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::types::GovernanceDecisionRow;

pub async fn list_decisions_filtered(
    pool: &PgPool,
    policy: Option<&str>,
    outcome: Option<&str>,
    user_id: Option<&UserId>,
    limit: i64,
) -> Result<Vec<GovernanceDecisionRow>, sqlx::Error> {
    let user_id = user_id.map(ToString::to_string);
    sqlx::query_as!(
        GovernanceDecisionRow,
        r#"SELECT id, user_id as "user_id!: _", tool_name,
                  agent_id as "agent_id: _", agent_scope,
                  decision, policy, reason, created_at
           FROM governance_decisions
           WHERE ($1::text IS NULL OR policy = $1)
             AND ($2::text IS NULL OR decision = $2)
             AND ($3::text IS NULL OR user_id = $3)
           ORDER BY created_at DESC
           LIMIT $4"#,
        policy,
        outcome,
        user_id,
        limit,
    )
    .fetch_all(pool)
    .await
}

// Why: lint-ok: unused-pub — live upstream via the ssr_governance handlers,
// which this fork does not ship; kept so shared repository files stay
// identical.
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
