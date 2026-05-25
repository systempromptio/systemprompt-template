//! Per-identity violation rollup for the heatmap and top-N lists.
//!
//! Returns one row per (identity, policy) where decision = 'deny'. Each row
//! carries `total_count` (allow + deny for the same pair) so callers can
//! render deny/total ratios. The identity dimension is dispatched at the
//! Rust layer rather than via string interpolation into SQL — every branch
//! is compile-time-verified by `sqlx::query!` against the live schema.

use serde::Serialize;
use sqlx::PgPool;

use super::time_range::TimeRange;

#[derive(Debug, Clone, Copy)]
pub enum IdentityGroupBy {
    User,
    Agent,
    AgentScope,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityViolations {
    pub identity_id: String,
    pub policy: String,
    pub deny_count: i64,
    pub total_count: i64,
}

pub async fn fetch_violations_by_identity(
    pool: &PgPool,
    range: TimeRange,
    group_by: IdentityGroupBy,
) -> Result<Vec<IdentityViolations>, sqlx::Error> {
    match group_by {
        IdentityGroupBy::User => sqlx::query!(
            r#"SELECT
                g.user_id AS "identity_id!",
                g.policy AS "policy!",
                COUNT(*) FILTER (WHERE g.decision = 'deny')::bigint AS "deny_count!",
                COUNT(*)::bigint AS "total_count!"
              FROM governance_decisions g
              WHERE g.created_at >= $1 AND g.created_at < $2
              GROUP BY g.user_id, g.policy
              HAVING COUNT(*) FILTER (WHERE g.decision = 'deny') > 0
              ORDER BY 3 DESC, 4 DESC
              LIMIT 500"#,
            range.from,
            range.to,
        )
        .fetch_all(pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|r| IdentityViolations {
                    identity_id: r.identity_id,
                    policy: r.policy,
                    deny_count: r.deny_count,
                    total_count: r.total_count,
                })
                .collect()
        }),
        IdentityGroupBy::Agent => sqlx::query!(
            r#"SELECT
                COALESCE(g.agent_id, '') AS "identity_id!",
                g.policy AS "policy!",
                COUNT(*) FILTER (WHERE g.decision = 'deny')::bigint AS "deny_count!",
                COUNT(*)::bigint AS "total_count!"
              FROM governance_decisions g
              WHERE g.created_at >= $1 AND g.created_at < $2
              GROUP BY COALESCE(g.agent_id, ''), g.policy
              HAVING COUNT(*) FILTER (WHERE g.decision = 'deny') > 0
              ORDER BY 3 DESC, 4 DESC
              LIMIT 500"#,
            range.from,
            range.to,
        )
        .fetch_all(pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|r| IdentityViolations {
                    identity_id: r.identity_id,
                    policy: r.policy,
                    deny_count: r.deny_count,
                    total_count: r.total_count,
                })
                .collect()
        }),
        IdentityGroupBy::AgentScope => sqlx::query!(
            r#"SELECT
                COALESCE(g.agent_scope, '') AS "identity_id!",
                g.policy AS "policy!",
                COUNT(*) FILTER (WHERE g.decision = 'deny')::bigint AS "deny_count!",
                COUNT(*)::bigint AS "total_count!"
              FROM governance_decisions g
              WHERE g.created_at >= $1 AND g.created_at < $2
              GROUP BY COALESCE(g.agent_scope, ''), g.policy
              HAVING COUNT(*) FILTER (WHERE g.decision = 'deny') > 0
              ORDER BY 3 DESC, 4 DESC
              LIMIT 500"#,
            range.from,
            range.to,
        )
        .fetch_all(pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|r| IdentityViolations {
                    identity_id: r.identity_id,
                    policy: r.policy,
                    deny_count: r.deny_count,
                    total_count: r.total_count,
                })
                .collect()
        }),
    }
}
