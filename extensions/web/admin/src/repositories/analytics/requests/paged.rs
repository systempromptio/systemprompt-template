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
use systemprompt_web_shared::{GroupId, ProjectId};

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
    group_id: Option<GroupId>,
    group_name: Option<String>,
    project_id: Option<ProjectId>,
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

    sqlx::query_file_as!(
        RequestRowWithTotal,
        "src/repositories/analytics/requests/paged.sql",
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
