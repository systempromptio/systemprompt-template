//! Paginated `ai_requests` listing with optional filters.
//!
//! Joins `users` for a friendly identity label and lateral subqueries on
//! `governance_decisions` and `plugin_usage_events` for per-row decision and
//! tool-call counts. The sort is a closed `RequestSortSpec`; each
//! `(column, dir)` pair is bound as text and selected by a per-key `CASE` in
//! the `ORDER BY`, keeping the whole statement a single `query_as!`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, AiRequestId, SessionId, TraceId, UserId};

use super::{RequestFilter, RequestRow, RequestSortSpec};
use crate::repositories::governance::time_range::TimeRange;

#[derive(Debug)]
struct RequestRowWithTotal {
    id: String,
    request_id: AiRequestId,
    created_at: DateTime<Utc>,
    user_id: UserId,
    user_label: Option<String>,
    session_id: Option<SessionId>,
    trace_id: Option<TraceId>,
    provider: String,
    model: String,
    status: String,
    input_tokens: Option<i32>,
    output_tokens: Option<i32>,
    cost_microdollars: i64,
    latency_ms: Option<i32>,
    error_message: Option<String>,
    decision_count: i64,
    deny_count: i64,
    tool_call_count: i64,
    total_count: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct RequestPage {
    pub sort: RequestSortSpec,
    pub limit: i64,
    pub offset: i64,
}

pub async fn fetch_requests_paged(
    pool: &PgPool,
    filter: &RequestFilter,
    range: TimeRange,
    page: RequestPage,
) -> Result<(Vec<RequestRow>, i64), sqlx::Error> {
    let search_pattern = filter
        .search
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));

    let rows = run_paged_query(pool, filter, range, page, search_pattern.as_deref()).await?;

    let total = rows.first().map_or(0, |r| r.total_count);
    let out = rows.into_iter().map(RequestRow::from).collect();
    Ok((out, total))
}

#[expect(
    clippy::too_many_lines,
    reason = "body is one irreducible compile-time-checked query_as! SQL literal"
)]
async fn run_paged_query(
    pool: &PgPool,
    filter: &RequestFilter,
    range: TimeRange,
    page: RequestPage,
    search_pattern: Option<&str>,
) -> Result<Vec<RequestRowWithTotal>, sqlx::Error> {
    let RequestPage {
        sort,
        limit,
        offset,
    } = page;
    let sort_col = sort.column.sql_key();
    let sort_dir = sort.dir.sql_key();

    sqlx::query_as!(
        RequestRowWithTotal,
        r#"WITH joined AS (
            SELECT
                ar.id, ar.request_id, ar.created_at, ar.user_id,
                ar.session_id, ar.trace_id, ar.provider, ar.model, ar.status,
                ar.input_tokens, ar.output_tokens,
                COALESCE(ar.cost_microdollars, 0)::bigint AS cost_microdollars,
                ar.cost_microdollars AS cost_raw,
                ar.latency_ms, ar.error_message,
                COALESCE(u.display_name, u.full_name, u.name, u.email) AS user_label,
                COALESCE((
                    SELECT COUNT(*)::bigint FROM governance_decisions g
                    WHERE g.session_id = ar.session_id
                ), 0) AS decision_count,
                COALESCE((
                    SELECT COUNT(*)::bigint FROM governance_decisions g
                    WHERE g.session_id = ar.session_id AND g.decision = 'deny'
                ), 0) AS deny_count,
                COALESCE((
                    SELECT COUNT(*)::bigint FROM plugin_usage_events e
                    WHERE e.session_id = ar.session_id
                      AND e.event_type ILIKE '%ToolUse%'
                ), 0) AS tool_call_count
            FROM ai_requests ar
            LEFT JOIN users u ON u.id = ar.user_id
            WHERE ar.created_at >= $1 AND ar.created_at < $2
              AND ($3::text IS NULL OR ar.user_id = $3)
              AND ($4::text IS NULL OR EXISTS (
                  SELECT 1 FROM governance_decisions g
                  WHERE g.session_id = ar.session_id AND g.agent_id = $4
              ))
              AND ($5::text IS NULL OR ar.model = $5)
              AND ($6::text IS NULL OR ar.provider = $6)
              AND ($7::text IS NULL OR ar.status = $7)
              AND ($8::text IS NULL
                   OR ar.user_id ILIKE $8
                   OR ar.model ILIKE $8
                   OR ar.provider ILIKE $8
                   OR COALESCE(ar.error_message, '') ILIKE $8
                   OR COALESCE(ar.trace_id, '') ILIKE $8)
        )
        SELECT
            id AS "id!",
            request_id AS "request_id!: AiRequestId",
            created_at AS "created_at!",
            user_id AS "user_id!: UserId",
            user_label,
            session_id AS "session_id: SessionId",
            trace_id AS "trace_id: TraceId",
            provider AS "provider!",
            model AS "model!",
            status AS "status!",
            input_tokens, output_tokens,
            cost_microdollars AS "cost_microdollars!",
            latency_ms, error_message,
            decision_count AS "decision_count!",
            deny_count AS "deny_count!",
            tool_call_count AS "tool_call_count!",
            (SELECT COUNT(*) FROM joined)::bigint AS "total_count!"
        FROM joined
        ORDER BY
            (CASE WHEN $11 = 'created_at' AND $12 = 'asc'  THEN created_at END) ASC  NULLS LAST,
            (CASE WHEN $11 = 'created_at' AND $12 = 'desc' THEN created_at END) DESC NULLS LAST,
            (CASE WHEN $11 = 'cost'    AND $12 = 'asc'  THEN cost_raw END) ASC  NULLS LAST,
            (CASE WHEN $11 = 'cost'    AND $12 = 'desc' THEN cost_raw END) DESC NULLS LAST,
            (CASE WHEN $11 = 'latency' AND $12 = 'asc'  THEN latency_ms END) ASC  NULLS LAST,
            (CASE WHEN $11 = 'latency' AND $12 = 'desc' THEN latency_ms END) DESC NULLS LAST,
            (CASE WHEN $11 = 'tokens'  AND $12 = 'asc'  THEN (COALESCE(input_tokens,0) + COALESCE(output_tokens,0)) END) ASC  NULLS LAST,
            (CASE WHEN $11 = 'tokens'  AND $12 = 'desc' THEN (COALESCE(input_tokens,0) + COALESCE(output_tokens,0)) END) DESC NULLS LAST
        LIMIT $9 OFFSET $10"#,
        range.from,
        range.to,
        filter.user_id.as_ref().map(UserId::as_str),
        filter.agent_id.as_ref().map(AgentId::as_str),
        filter.model.as_deref(),
        filter.provider.as_deref(),
        filter.status.as_deref(),
        search_pattern,
        limit,
        offset,
        sort_col,
        sort_dir,
    )
    .fetch_all(pool)
    .await
}

impl From<RequestRowWithTotal> for RequestRow {
    fn from(r: RequestRowWithTotal) -> Self {
        Self {
            id: r.id,
            request_id: r.request_id,
            created_at: r.created_at,
            user_id: r.user_id,
            user_label: r.user_label,
            session_id: r.session_id,
            trace_id: r.trace_id,
            provider: r.provider,
            model: r.model,
            status: r.status,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cost_microdollars: r.cost_microdollars,
            latency_ms: r.latency_ms,
            error_message: r.error_message,
            decision_count: r.decision_count,
            deny_count: r.deny_count,
            tool_call_count: r.tool_call_count,
        }
    }
}
