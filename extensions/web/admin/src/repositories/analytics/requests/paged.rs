//! Paginated `ai_requests` listing with optional filters.
//!
//! Joins `users` for a friendly identity label, `user_scope_defaults` (plus
//! `groups` / `projects`) for the exclusive attribution the log's Project and
//! Group columns show, and lateral subqueries on `governance_decisions` and
//! `ai_request_tool_calls` for per-row decision and tool-call counts. The sort
//! is a closed `RequestSortSpec`; each `(column, dir)` pair is bound as text
//! and selected by a per-key `CASE` in the `ORDER BY`, keeping the whole
//! statement a single `query_as!`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, AiRequestId, SessionId, TraceId, UserId};

use super::{RequestFilter, RequestRow, RequestSortSpec};
use crate::util::time_range::TimeRange;

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
    group_id: Option<String>,
    group_name: Option<String>,
    project_id: Option<String>,
    project_name: Option<String>,
    total_count: i64,
}

/// Pagination window for [`list_requests_paged`]: LIMIT/OFFSET plus the
/// closed sort spec, grouped since callers always pass all three together.
#[derive(Debug, Clone, Copy)]
pub struct RequestPage {
    pub sort: RequestSortSpec,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list_requests_paged(
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
                ar.session_id, ar.trace_id,
                -- Why: a request rejected before route resolution has no
                -- provider or model. Both columns are nullable, and the outer
                -- SELECT asserts them non-null, so one such row anywhere in a
                -- page used to fail the decode and blank the whole page.
                COALESCE(ar.provider, '') AS provider,
                COALESCE(ar.model, '') AS model,
                ar.status,
                ar.input_tokens, ar.output_tokens,
                COALESCE(ar.cost_microdollars, 0)::bigint AS cost_microdollars,
                ar.cost_microdollars AS cost_raw,
                ar.latency_ms, ar.error_message,
                COALESCE(u.display_name, u.full_name, u.name, u.email) AS user_label,
                sd.primary_group_id AS group_id,
                g.name AS group_name,
                sd.primary_project_id AS project_id,
                p.name AS project_name,
                COALESCE((
                    SELECT COUNT(*)::bigint FROM governance_decisions gd
                    WHERE gd.session_id = ar.session_id
                ), 0) AS decision_count,
                COALESCE((
                    SELECT COUNT(*)::bigint FROM governance_decisions gd
                    WHERE gd.session_id = ar.session_id AND gd.decision = 'deny'
                ), 0) AS deny_count,
                COALESCE((
                    SELECT COUNT(*)::bigint FROM ai_request_tool_calls tc
                    WHERE tc.request_id = ar.id
                ), 0) AS tool_call_count
            FROM ai_requests ar
            LEFT JOIN users u ON u.id = ar.user_id
            -- Why: exclusive attribution. A person counts once, against the
            -- one group and the one project `user_scope_defaults` names, so
            -- the log's totals cannot double-count a multi-group member.
            LEFT JOIN user_scope_defaults sd ON sd.user_id = ar.user_id
            LEFT JOIN groups g ON g.id = sd.primary_group_id
            LEFT JOIN projects p ON p.id = sd.primary_project_id
            WHERE ar.created_at >= $1 AND ar.created_at < $2
              AND ($13::TEXT[] IS NULL OR ar.user_id = ANY($13))
              AND ($3::text IS NULL OR ar.user_id = $3)
              AND ($4::text IS NULL OR EXISTS (
                  SELECT 1 FROM governance_decisions gd
                  WHERE gd.session_id = ar.session_id AND gd.agent_id = $4
              ))
              -- Why: a call the gateway rejected before it resolved a route has
              -- no model and no provider. `unrouted` is the sentinel the
              -- analytics drill-downs send for exactly those rows, so it has
              -- to mean IS NULL rather than a model literally named that.
              AND ($5::text IS NULL
                   OR ($5 = 'unrouted' AND ar.model IS NULL)
                   OR ar.model = $5)
              AND ($6::text IS NULL
                   OR ($6 = 'unrouted' AND ar.provider IS NULL)
                   OR ar.provider = $6)
              AND ($7::text IS NULL OR ar.status = $7)
              AND ($14::text IS NULL OR EXISTS (
                  SELECT 1 FROM ai_request_tool_calls tc
                  WHERE tc.request_id = ar.id AND tc.tool_name = $14
              ))
              AND ($15::text IS NULL OR sd.primary_group_id = $15)
              AND ($16::text IS NULL OR sd.primary_project_id = $16)
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
            -- Why: these four reach the row through LEFT JOINs, so every one of
            -- them is null for a person the directory placed nowhere. sqlx
            -- infers nullability from the source column, and `groups.name` and
            -- `projects.name` are NOT NULL in their own tables, so without the
            -- `?` override the macro generates a non-Option decode and one
            -- unattributed row anywhere in the page fails the whole query.
            group_id AS "group_id?", group_name AS "group_name?",
            project_id AS "project_id?", project_name AS "project_name?",
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
        filter.scope.as_sql(),
        filter.tool.as_deref(),
        filter.group.as_deref(),
        filter.project.as_deref(),
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
            group_id: r.group_id,
            group_name: r.group_name,
            project_id: r.project_id,
            project_name: r.project_name,
        }
    }
}
