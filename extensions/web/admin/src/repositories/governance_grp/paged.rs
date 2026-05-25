//! Paginated decisions listing for the audit explorer page.

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

impl SortSpec {
    const fn order_by(self) -> &'static str {
        match (self.column, self.dir) {
            (SortColumn::CreatedAt, SortDir::Asc) => "g.created_at ASC",
            (SortColumn::CreatedAt, SortDir::Desc) => "g.created_at DESC",
            (SortColumn::Cost, SortDir::Asc) => "ar.cost_microdollars ASC NULLS LAST",
            (SortColumn::Cost, SortDir::Desc) => "ar.cost_microdollars DESC NULLS LAST",
            (SortColumn::Latency, SortDir::Asc) => "ar.latency_ms ASC NULLS LAST",
            (SortColumn::Latency, SortDir::Desc) => "ar.latency_ms DESC NULLS LAST",
            (SortColumn::Policy, SortDir::Asc) => "g.policy ASC",
            (SortColumn::Policy, SortDir::Desc) => "g.policy DESC",
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

/// Fetch a page of governance decisions with `ai_requests` enrichment.
///
/// LEFT JOINs `ai_requests` on `session_id` (the only column shared by both
/// tables) to surface `cost_microdollars`, `latency_ms`, and `trace_id`.
/// Returns `(rows, total_count)`.
pub async fn fetch_decisions_paged(
    pool: &PgPool,
    filter: &DecisionFilter,
    range: TimeRange,
    sort: SortSpec,
    limit: i64,
    offset: i64,
) -> Result<(Vec<DecisionRow>, i64), sqlx::Error> {
    let search_pattern = filter
        .search
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));

    let order = sort.order_by();
    let sql = format!(
        r"WITH joined AS (
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
            id, user_id, session_id, tool_name, agent_id, agent_scope,
            decision, policy, reason,
            COALESCE(evaluated_rules, '[]'::jsonb) as evaluated_rules,
            plugin_id, trace_id, cost_microdollars, latency_ms, created_at,
            (SELECT COUNT(*) FROM joined)::bigint AS total_count
        FROM joined
        ORDER BY {order}
        LIMIT $10 OFFSET $11",
    );

    // Why: {order} is built from the GovernanceDecisionSort closed enum
    // (sort.order_by()) and interpolates a multi-column ORDER BY expression
    // that PG cannot parameterise.
    let rows = sqlx::query_as::<_, PagedRow>(&sql)
        .bind(range.from)
        .bind(range.to)
        .bind(filter.user_id.as_deref())
        .bind(filter.agent_id.as_deref())
        .bind(filter.agent_scope)
        .bind(filter.policy.as_deref())
        .bind(filter.decision.as_deref())
        .bind(filter.tool_name.as_deref())
        .bind(search_pattern.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total = rows.first().map_or(0, |r| r.total_count);
    let decisions = rows
        .into_iter()
        .map(|r| DecisionRow {
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
        })
        .collect();

    Ok((decisions, total))
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
