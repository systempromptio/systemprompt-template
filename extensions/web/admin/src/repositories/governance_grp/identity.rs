//! Per-identity violation rollup for the heatmap and top-N lists.

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

/// Returns one row per (identity, policy) where decision = 'deny'.
///
/// Each row also carries `total_count` (allow + deny for the same pair) so the
/// caller can render deny/total ratios in the heatmap.
pub async fn fetch_violations_by_identity(
    pool: &PgPool,
    range: TimeRange,
    group_by: IdentityGroupBy,
) -> Result<Vec<IdentityViolations>, sqlx::Error> {
    let identity_expr = match group_by {
        IdentityGroupBy::User => "g.user_id",
        IdentityGroupBy::Agent => "COALESCE(g.agent_id, '')",
        IdentityGroupBy::AgentScope => "COALESCE(g.agent_scope, '')",
    };

    let sql = format!(
        r"SELECT
            {identity_expr} AS identity_id,
            g.policy AS policy,
            COUNT(*) FILTER (WHERE g.decision = 'deny')::bigint AS deny_count,
            COUNT(*)::bigint AS total_count
           FROM governance_decisions g
           WHERE g.created_at >= $1 AND g.created_at < $2
           GROUP BY identity_id, g.policy
           HAVING COUNT(*) FILTER (WHERE g.decision = 'deny') > 0
           ORDER BY deny_count DESC, total_count DESC
           LIMIT 500",
    );

    sqlx::query_as::<_, IdentityViolationsRow>(&sql)
        .bind(range.from)
        .bind(range.to)
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
        })
}

#[derive(sqlx::FromRow)]
struct IdentityViolationsRow {
    identity_id: String,
    policy: String,
    deny_count: i64,
    total_count: i64,
}
