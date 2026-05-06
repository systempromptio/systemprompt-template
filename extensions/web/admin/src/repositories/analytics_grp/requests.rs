use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::governance_grp::time_range::TimeRange;

#[derive(Debug, Clone, Copy)]
pub struct RequestStatsRow {
    pub total: i64,
    pub tool_uses: i64,
    pub errors: i64,
    pub sessions: i64,
}

#[derive(Debug, Clone)]
pub struct RecentGatewayRequestRow {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub provider: String,
    pub model: String,
    pub status: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost_microdollars: i64,
    pub latency_ms: Option<i32>,
    pub trace_id: Option<String>,
    pub error_message: Option<String>,
}

pub async fn list_recent_gateway_requests(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<RecentGatewayRequestRow>, sqlx::Error> {
    sqlx::query_as!(
        RecentGatewayRequestRow,
        r#"SELECT
            id as "id!",
            created_at as "created_at!",
            provider as "provider!",
            model as "model!",
            status as "status!",
            input_tokens,
            output_tokens,
            COALESCE(cost_microdollars, 0)::bigint as "cost_microdollars!",
            latency_ms,
            trace_id,
            error_message
          FROM ai_requests
          ORDER BY created_at DESC
          LIMIT $1"#,
        limit,
    )
    .fetch_all(pool)
    .await
}

/// Legacy stats query (24h window over `plugin_usage_events`).
///
/// Kept for backward compatibility with callers we haven't migrated yet — the
/// new Inference Requests page uses [`request_stats::fetch_request_stats`].
pub async fn get_request_stats(pool: &PgPool) -> Result<RequestStatsRow, sqlx::Error> {
    sqlx::query_as!(
        RequestStatsRow,
        r#"SELECT
            COUNT(*)::bigint AS "total!",
            COUNT(*) FILTER (WHERE event_type LIKE '%ToolUse%')::bigint AS "tool_uses!",
            COUNT(*) FILTER (WHERE event_type LIKE '%Failure%' OR event_type LIKE '%Error%')::bigint AS "errors!",
            COUNT(DISTINCT session_id)::bigint AS "sessions!"
          FROM plugin_usage_events
          WHERE created_at >= NOW() - INTERVAL '24 hours'"#,
    )
    .fetch_one(pool)
    .await
}

#[derive(Debug, Clone, Default)]
pub struct RequestFilter {
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum RequestSortColumn {
    CreatedAt,
    Cost,
    Latency,
    Tokens,
}

#[derive(Debug, Clone, Copy)]
pub enum SortDir {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy)]
pub struct RequestSortSpec {
    pub column: RequestSortColumn,
    pub dir: SortDir,
}

impl Default for RequestSortSpec {
    fn default() -> Self {
        Self {
            column: RequestSortColumn::CreatedAt,
            dir: SortDir::Desc,
        }
    }
}

impl RequestSortSpec {
    const fn order_by(self) -> &'static str {
        match (self.column, self.dir) {
            (RequestSortColumn::CreatedAt, SortDir::Asc) => "ar.created_at ASC",
            (RequestSortColumn::CreatedAt, SortDir::Desc) => "ar.created_at DESC",
            (RequestSortColumn::Cost, SortDir::Asc) => "ar.cost_microdollars ASC NULLS LAST",
            (RequestSortColumn::Cost, SortDir::Desc) => "ar.cost_microdollars DESC NULLS LAST",
            (RequestSortColumn::Latency, SortDir::Asc) => "ar.latency_ms ASC NULLS LAST",
            (RequestSortColumn::Latency, SortDir::Desc) => "ar.latency_ms DESC NULLS LAST",
            (RequestSortColumn::Tokens, SortDir::Asc) => {
                "(COALESCE(ar.input_tokens,0)+COALESCE(ar.output_tokens,0)) ASC"
            }
            (RequestSortColumn::Tokens, SortDir::Desc) => {
                "(COALESCE(ar.input_tokens,0)+COALESCE(ar.output_tokens,0)) DESC"
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestRow {
    pub id: String,
    pub request_id: String,
    pub created_at: DateTime<Utc>,
    pub user_id: String,
    pub user_label: Option<String>,
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
    pub provider: String,
    pub model: String,
    pub status: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost_microdollars: i64,
    pub latency_ms: Option<i32>,
    pub error_message: Option<String>,
    pub decision_count: i64,
    pub deny_count: i64,
    pub tool_call_count: i64,
}

/// Page through `ai_requests` with optional filters.
///
/// Joins `users` for a friendly identity label and lateral subqueries on
/// `governance_decisions` and `plugin_usage_events` for per-row decision and
/// tool-call counts.
pub async fn fetch_requests_paged(
    pool: &PgPool,
    filter: &RequestFilter,
    range: TimeRange,
    sort: RequestSortSpec,
    limit: i64,
    offset: i64,
) -> Result<(Vec<RequestRow>, i64), sqlx::Error> {
    let search_pattern = filter
        .search
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));

    let order = sort.order_by();
    let sql = format!(
        r"WITH joined AS (
            SELECT
                ar.id, ar.request_id, ar.created_at, ar.user_id,
                ar.session_id, ar.trace_id, ar.provider, ar.model, ar.status,
                ar.input_tokens, ar.output_tokens,
                COALESCE(ar.cost_microdollars, 0)::bigint AS cost_microdollars,
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
            id, request_id, created_at, user_id, user_label,
            session_id, trace_id, provider, model, status,
            input_tokens, output_tokens, cost_microdollars, latency_ms,
            error_message, decision_count, deny_count, tool_call_count,
            (SELECT COUNT(*) FROM joined)::bigint AS total_count
        FROM joined
        ORDER BY {order}
        LIMIT $9 OFFSET $10",
    );

    let rows = sqlx::query_as::<_, RequestRowWithTotal>(&sql)
        .bind(range.from)
        .bind(range.to)
        .bind(filter.user_id.as_deref())
        .bind(filter.agent_id.as_deref())
        .bind(filter.model.as_deref())
        .bind(filter.provider.as_deref())
        .bind(filter.status.as_deref())
        .bind(search_pattern.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total = rows.first().map_or(0, |r| r.total_count);
    let out = rows
        .into_iter()
        .map(|r| RequestRow {
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
        })
        .collect();
    Ok((out, total))
}

#[derive(sqlx::FromRow)]
struct RequestRowWithTotal {
    id: String,
    request_id: String,
    created_at: DateTime<Utc>,
    user_id: String,
    user_label: Option<String>,
    session_id: Option<String>,
    trace_id: Option<String>,
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
