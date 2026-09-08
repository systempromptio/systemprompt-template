//! The paged governance decision log: one row per evaluated tool call.
//!
//! The rollups in [`super::warnings`] answer "how much"; this answers "which
//! call, and why". Both read `governance_decisions`, and they are separate
//! because a rollup that also had to be paginated would have to choose between
//! being complete and being a page, and the log needs to be both.
//!
//! Sorting is expressed as ordered `CASE` arms rather than as interpolated SQL
//! because `sqlx::query_as!` verifies static text only: a sort key spliced into
//! the string would be unverified and, being caller-supplied, an injection
//! site. The arms cost one comparison per row and keep the whole statement
//! checked at compile time.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::scope::SubjectScope;
use crate::util::time_range::TimeRange;

/// One evaluated call, as the decisions tab renders it.
#[derive(Debug, Clone)]
pub struct DecisionLogRow {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub decision: String,
    pub policy: String,
    pub tool_name: String,
    pub user_id: UserId,
    pub agent_scope: Option<String>,
    pub reason: String,
    pub trace_id: Option<String>,
}

/// The KPI strip's numbers, over the same window and scope as the rows.
#[derive(Debug, Clone, Copy, Default)]
pub struct DecisionStats {
    pub evaluated: i64,
    pub allowed: i64,
    pub warned: i64,
    pub denied: i64,
    pub scope_denied: i64,
    pub secret_denied: i64,
    pub blocklist_denied: i64,
    pub rate_denied: i64,
    pub distinct_users: i64,
}

/// What the toolbar narrowed the log to.
#[derive(Debug, Clone, Default)]
pub struct DecisionFilter {
    pub policy: Option<String>,
    pub decision: Option<String>,
    pub search: Option<String>,
}

/// Column and direction the log is ordered by.
#[derive(Debug, Clone, Copy)]
pub struct DecisionSort {
    pub key: &'static str,
    pub ascending: bool,
}

impl Default for DecisionSort {
    fn default() -> Self {
        Self {
            key: "when",
            ascending: false,
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "page query plumbing; splitting the parameters is tracked in docs/tech-debt.md"
)]
pub async fn list_governance_decisions_paged(
    pool: &PgPool,
    range: TimeRange,
    scope: &SubjectScope,
    filter: &DecisionFilter,
    sort: DecisionSort,
    limit: i64,
    offset: i64,
) -> Result<(Vec<DecisionLogRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as!(
        DecisionLogRow,
        r#"SELECT g.id, g.created_at, g.decision, g.policy, g.tool_name,
                  g.user_id AS "user_id!: UserId", g.agent_scope, g.reason, g.trace_id
           FROM governance_decisions g
           WHERE g.created_at >= $1 AND g.created_at < $2
             AND ($3::TEXT[] IS NULL OR g.user_id = ANY($3))
             AND ($4::TEXT IS NULL OR g.policy = $4)
             AND ($5::TEXT IS NULL OR g.decision = $5)
             AND ($6::TEXT IS NULL
                  OR g.tool_name ILIKE '%' || $6 || '%'
                  OR g.user_id ILIKE '%' || $6 || '%'
                  OR g.reason ILIKE '%' || $6 || '%')
           ORDER BY
             CASE WHEN $7 = 'policy'   AND $8 THEN g.policy END ASC,
             CASE WHEN $7 = 'policy'   AND NOT $8 THEN g.policy END DESC,
             CASE WHEN $7 = 'decision' AND $8 THEN g.decision END ASC,
             CASE WHEN $7 = 'decision' AND NOT $8 THEN g.decision END DESC,
             CASE WHEN $7 = 'tool'     AND $8 THEN g.tool_name END ASC,
             CASE WHEN $7 = 'tool'     AND NOT $8 THEN g.tool_name END DESC,
             CASE WHEN $7 = 'user'     AND $8 THEN g.user_id END ASC,
             CASE WHEN $7 = 'user'     AND NOT $8 THEN g.user_id END DESC,
             CASE WHEN $8 THEN g.created_at END ASC,
             g.created_at DESC
           LIMIT $9 OFFSET $10"#,
        range.from,
        range.to,
        scope.as_sql(),
        filter.policy.as_deref(),
        filter.decision.as_deref(),
        filter.search.as_deref(),
        sort.key,
        sort.ascending,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*)::BIGINT AS "n!"
           FROM governance_decisions g
           WHERE g.created_at >= $1 AND g.created_at < $2
             AND ($3::TEXT[] IS NULL OR g.user_id = ANY($3))
             AND ($4::TEXT IS NULL OR g.policy = $4)
             AND ($5::TEXT IS NULL OR g.decision = $5)
             AND ($6::TEXT IS NULL
                  OR g.tool_name ILIKE '%' || $6 || '%'
                  OR g.user_id ILIKE '%' || $6 || '%'
                  OR g.reason ILIKE '%' || $6 || '%')"#,
        range.from,
        range.to,
        scope.as_sql(),
        filter.policy.as_deref(),
        filter.decision.as_deref(),
        filter.search.as_deref(),
    )
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

// Why: the four per-policy counts are the four synchronous chain stages, named
// as the policy writes them. They are counted here rather than derived from the
// page of rows because a page is fifty rows and the deny rate is a property of
// the window.
pub async fn get_decision_stats(
    pool: &PgPool,
    range: TimeRange,
    scope: &SubjectScope,
) -> Result<DecisionStats, sqlx::Error> {
    sqlx::query_as!(
        DecisionStats,
        r#"SELECT
             COUNT(*)::BIGINT AS "evaluated!",
             COUNT(*) FILTER (WHERE decision = 'allow')::BIGINT AS "allowed!",
             COUNT(*) FILTER (WHERE decision = 'warn')::BIGINT AS "warned!",
             COUNT(*) FILTER (WHERE decision = 'deny')::BIGINT AS "denied!",
             COUNT(*) FILTER (WHERE decision = 'deny' AND policy = 'agent_scope')::BIGINT
               AS "scope_denied!",
             COUNT(*) FILTER (WHERE decision = 'deny' AND policy = 'secret_scan')::BIGINT
               AS "secret_denied!",
             COUNT(*) FILTER (WHERE decision = 'deny' AND policy = 'tool_blocklist')::BIGINT
               AS "blocklist_denied!",
             COUNT(*) FILTER (WHERE decision = 'deny' AND policy = 'rate_limit')::BIGINT
               AS "rate_denied!",
             COUNT(DISTINCT user_id)::BIGINT AS "distinct_users!"
           FROM governance_decisions
           WHERE created_at >= $1 AND created_at < $2
             AND ($3::TEXT[] IS NULL OR user_id = ANY($3))"#,
        range.from,
        range.to,
        scope.as_sql(),
    )
    .fetch_one(pool)
    .await
}

// Why: built from the window rather than from a fixed list, so a policy
// that stops firing stops being offered as a filter.
pub async fn list_decision_policies(
    pool: &PgPool,
    range: TimeRange,
    scope: &SubjectScope,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT policy AS "policy!"
           FROM governance_decisions
           WHERE created_at >= $1 AND created_at < $2
             AND ($3::TEXT[] IS NULL OR user_id = ANY($3))
           GROUP BY policy
           ORDER BY COUNT(*) DESC, policy"#,
        range.from,
        range.to,
        scope.as_sql(),
    )
    .fetch_all(pool)
    .await
}
