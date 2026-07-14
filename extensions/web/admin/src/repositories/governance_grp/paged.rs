//! Paginated decisions listing for the audit explorer page.
//!
//! `DecisionRow` and `PagedRow` deliberately surface `evaluated_rules` as
//! untyped JSON: the audit blob is typed (`DecisionAudit`) on the writing
//! side, and the dashboard renders it generically here. Reintroducing the
//! typed shape at the read boundary would couple the explorer to every
//! `DecisionAudit` schema change for no rendering benefit.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt_security::policy::types::AccessScope;

use super::time_range::TimeRange;

#[derive(Debug, Clone, Default)]
pub struct DecisionFilter {
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub agent_scope: Option<AccessScope>,
    pub policy: Option<String>,
    pub decision: Option<String>,
    pub tool_name: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum SortColumn {
    CreatedAt,
    Cost,
    Latency,
    Policy,
}

#[derive(Debug, Clone, Copy)]
pub enum SortDir {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy)]
pub struct SortSpec {
    pub column: SortColumn,
    pub dir: SortDir,
}

impl Default for SortSpec {
    fn default() -> Self {
        Self {
            column: SortColumn::CreatedAt,
            dir: SortDir::Desc,
        }
    }
}

impl SortColumn {
    const fn sql_key(self) -> &'static str {
        match self {
            Self::CreatedAt => "created_at",
            Self::Cost => "cost",
            Self::Latency => "latency",
            Self::Policy => "policy",
        }
    }
}

impl SortDir {
    const fn sql_key(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DecisionRow {
    pub id: String,
    pub user_id: String,
    pub session_id: String,
    pub tool_name: String,
    pub agent_id: Option<String>,
    pub agent_scope: Option<AccessScope>,
    pub decision: String,
    pub policy: String,
    pub reason: String,
    pub evaluated_rules: serde_json::Value,
    pub plugin_id: Option<String>,
    pub trace_id: Option<String>,
    pub cost_microdollars: Option<i64>,
    pub latency_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Pagination window for [`fetch_decisions_paged`] (was 3 trailing positional
/// args: sort, limit, offset).
#[derive(Debug, Clone, Copy)]
pub struct DecisionPage {
    pub sort: SortSpec,
    pub limit: i64,
    pub offset: i64,
}

pub async fn fetch_decisions_paged(
    pool: &PgPool,
    filter: &DecisionFilter,
    range: TimeRange,
    page: DecisionPage,
) -> Result<(Vec<DecisionRow>, i64), sqlx::Error> {
    let search_pattern = filter
        .search
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));

    let rows = run_decisions_query(pool, filter, range, page, search_pattern.as_deref()).await?;

    let total = rows.first().map_or(0, |r| r.total_count);
    let decisions = rows.into_iter().map(DecisionRow::from).collect();

    Ok((decisions, total))
}

#[expect(
    clippy::too_many_lines,
    reason = "body is one irreducible compile-time-checked query_as! SQL literal"
)]
async fn run_decisions_query(
    pool: &PgPool,
    filter: &DecisionFilter,
    range: TimeRange,
    page: DecisionPage,
    search_pattern: Option<&str>,
) -> Result<Vec<PagedRow>, sqlx::Error> {
    let DecisionPage {
        sort,
        limit,
        offset,
    } = page;
    let sort_col = sort.column.sql_key();
    let sort_dir = sort.dir.sql_key();
    let agent_scope = filter.agent_scope.as_ref().map(|s| s.as_str());

    sqlx::query_as!(
        PagedRow,
        r#"WITH joined AS (
            SELECT
                g.id, g.user_id, g.session_id, g.tool_name, g.agent_id, g.agent_scope,
                g.decision, g.policy, g.reason, g.evaluated_rules, g.plugin_id,
                g.created_at,
                ar.trace_id, ar.cost_microdollars, ar.latency_ms
            FROM governance_decisions g
            LEFT JOIN LATERAL (
                SELECT trace_id, cost_microdollars, latency_ms
                FROM ai_requests ar2
                WHERE ar2.session_id = g.session_id
                ORDER BY ar2.created_at DESC
                LIMIT 1
            ) ar ON TRUE
            WHERE g.created_at >= $1 AND g.created_at < $2
              AND ($3::text IS NULL OR g.user_id = $3)
              AND ($4::text IS NULL OR g.agent_id = $4)
              AND ($5::text IS NULL OR g.agent_scope = $5)
              AND ($6::text IS NULL OR g.policy = $6)
              AND ($7::text IS NULL OR g.decision = $7)
              AND ($8::text IS NULL OR g.tool_name = $8)
              AND ($9::text IS NULL
                   OR g.user_id ILIKE $9
                   OR g.tool_name ILIKE $9
                   OR g.policy ILIKE $9
                   OR g.reason ILIKE $9
                   OR COALESCE(g.agent_id, '') ILIKE $9
                   OR COALESCE(g.agent_scope, '') ILIKE $9)
        )
        SELECT
            id AS "id!",
            user_id AS "user_id!",
            session_id AS "session_id!",
            tool_name AS "tool_name!",
            agent_id,
            agent_scope AS "agent_scope: AccessScope",
            decision AS "decision!",
            policy AS "policy!",
            reason AS "reason!",
            COALESCE(evaluated_rules, '[]'::jsonb) AS "evaluated_rules!",
            plugin_id, trace_id, cost_microdollars, latency_ms,
            created_at AS "created_at!",
            (SELECT COUNT(*) FROM joined)::bigint AS "total_count!"
        FROM joined
        ORDER BY
            (CASE WHEN $12 = 'created_at' AND $13 = 'asc'  THEN created_at END) ASC  NULLS LAST,
            (CASE WHEN $12 = 'created_at' AND $13 = 'desc' THEN created_at END) DESC NULLS LAST,
            (CASE WHEN $12 = 'cost'    AND $13 = 'asc'  THEN cost_microdollars END) ASC  NULLS LAST,
            (CASE WHEN $12 = 'cost'    AND $13 = 'desc' THEN cost_microdollars END) DESC NULLS LAST,
            (CASE WHEN $12 = 'latency' AND $13 = 'asc'  THEN latency_ms END) ASC  NULLS LAST,
            (CASE WHEN $12 = 'latency' AND $13 = 'desc' THEN latency_ms END) DESC NULLS LAST,
            (CASE WHEN $12 = 'policy'  AND $13 = 'asc'  THEN policy END) ASC  NULLS LAST,
            (CASE WHEN $12 = 'policy'  AND $13 = 'desc' THEN policy END) DESC NULLS LAST
        LIMIT $10 OFFSET $11"#,
        range.from,
        range.to,
        filter.user_id.as_deref(),
        filter.agent_id.as_deref(),
        agent_scope,
        filter.policy.as_deref(),
        filter.decision.as_deref(),
        filter.tool_name.as_deref(),
        search_pattern,
        limit,
        offset,
        sort_col,
        sort_dir,
    )
    .fetch_all(pool)
    .await
}

#[derive(sqlx::FromRow)]
struct PagedRow {
    id: String,
    user_id: String,
    session_id: String,
    tool_name: String,
    agent_id: Option<String>,
    agent_scope: Option<AccessScope>,
    decision: String,
    policy: String,
    reason: String,
    evaluated_rules: serde_json::Value,
    plugin_id: Option<String>,
    trace_id: Option<String>,
    cost_microdollars: Option<i64>,
    latency_ms: Option<i32>,
    created_at: DateTime<Utc>,
    total_count: i64,
}

impl From<PagedRow> for DecisionRow {
    fn from(r: PagedRow) -> Self {
        Self {
            id: r.id,
            user_id: r.user_id,
            session_id: r.session_id,
            tool_name: r.tool_name,
            agent_id: r.agent_id,
            agent_scope: r.agent_scope,
            decision: r.decision,
            policy: r.policy,
            reason: r.reason,
            evaluated_rules: r.evaluated_rules,
            plugin_id: r.plugin_id,
            trace_id: r.trace_id,
            cost_microdollars: r.cost_microdollars,
            latency_ms: r.latency_ms,
            created_at: r.created_at,
        }
    }
}
