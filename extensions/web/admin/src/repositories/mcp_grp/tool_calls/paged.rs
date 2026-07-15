//! Paginated tool-call listing with governance verdict and parent `trace_id`.
//!
//! The sort is a closed `ToolSortSpec`; each `(column, dir)` pair is bound as
//! text and selected by a per-key `CASE` in the `ORDER BY`, so the statement
//! stays a single compile-time `query_as!`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, SessionId, UserId};

use super::{ToolCallFilter, ToolCallRow, ToolSortSpec};
use crate::repositories::governance_grp::time_range::TimeRange;

#[derive(Debug)]
struct ToolCallRowWithTotal {
    id: String,
    created_at: DateTime<Utc>,
    event_type: String,
    tool_name: Option<String>,
    plugin_id: Option<String>,
    user_id: UserId,
    session_id: SessionId,
    agent_id: Option<AgentId>,
    agent_scope: Option<String>,
    content_input_bytes: i64,
    content_output_bytes: i64,
    decision: Option<String>,
    policy: Option<String>,
    reason: Option<String>,
    trace_id: Option<String>,
    ar_latency_ms: Option<i32>,
    metadata: serde_json::Value,
    total_count: i64,
}

/// Pagination window for [`fetch_tool_calls_paged`] (was 3 trailing
/// positional args: sort, limit, offset).
#[derive(Debug, Clone, Copy)]
pub struct ToolCallPage {
    pub sort: ToolSortSpec,
    pub limit: i64,
    pub offset: i64,
}

/// Page through tool-call events with their governance verdict and parent
/// request `trace_id`. Filter / sort / search supported.
pub async fn fetch_tool_calls_paged(
    pool: &PgPool,
    filter: &ToolCallFilter,
    range: TimeRange,
    page: ToolCallPage,
) -> Result<(Vec<ToolCallRow>, i64), sqlx::Error> {
    let search_pattern = filter
        .search
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));

    let rows = run_tool_calls_query(pool, filter, range, page, search_pattern.as_deref()).await?;

    let total = rows.first().map_or(0, |r| r.total_count);
    let out = rows.into_iter().map(ToolCallRow::from).collect();
    Ok((out, total))
}

#[expect(
    clippy::too_many_lines,
    reason = "body is one irreducible compile-time-checked query_as! SQL literal"
)]
async fn run_tool_calls_query(
    pool: &PgPool,
    filter: &ToolCallFilter,
    range: TimeRange,
    page: ToolCallPage,
    search_pattern: Option<&str>,
) -> Result<Vec<ToolCallRowWithTotal>, sqlx::Error> {
    let ToolCallPage {
        sort,
        limit,
        offset,
    } = page;
    let sort_col = sort.column.sql_key();
    let sort_dir = sort.dir.sql_key();

    sqlx::query_as!(
        ToolCallRowWithTotal,
        r#"WITH joined AS (
            SELECT
                e.id, e.created_at, e.event_type, e.tool_name, e.plugin_id,
                e.user_id, e.session_id,
                COALESCE(e.content_input_bytes, 0)::bigint AS content_input_bytes,
                COALESCE(e.content_output_bytes, 0)::bigint AS content_output_bytes,
                COALESCE(e.metadata, '{}'::jsonb) AS metadata,
                gd.decision, gd.policy, gd.reason, gd.agent_id, gd.agent_scope,
                ar.trace_id AS trace_id,
                ar.latency_ms AS ar_latency_ms
            FROM plugin_usage_events e
            LEFT JOIN LATERAL (
                SELECT decision, policy, reason, agent_id, agent_scope
                FROM governance_decisions g
                WHERE g.session_id = e.session_id
                  AND (g.tool_name = e.tool_name OR e.tool_name IS NULL)
                ORDER BY g.created_at DESC
                LIMIT 1
            ) gd ON TRUE
            LEFT JOIN LATERAL (
                SELECT trace_id, latency_ms
                FROM ai_requests ar2
                WHERE ar2.session_id = e.session_id
                ORDER BY ar2.created_at DESC
                LIMIT 1
            ) ar ON TRUE
            WHERE e.created_at >= $1 AND e.created_at < $2
              AND e.event_type ILIKE '%ToolUse%'
              AND ($3::text IS NULL OR e.tool_name = $3)
              AND ($4::text IS NULL OR e.user_id = $4)
              AND ($5::text IS NULL OR gd.agent_scope = $5)
              AND ($6::text IS NULL OR e.plugin_id = $6)
              AND ($7::text IS NULL OR gd.decision = $7)
              AND ($8::text IS NULL
                   OR e.user_id ILIKE $8
                   OR COALESCE(e.tool_name, '') ILIKE $8
                   OR COALESCE(e.plugin_id, '') ILIKE $8
                   OR COALESCE(gd.reason, '') ILIKE $8)
        )
        SELECT
            id AS "id!",
            created_at AS "created_at!",
            event_type AS "event_type!",
            tool_name,
            plugin_id,
            user_id AS "user_id!: UserId",
            session_id AS "session_id!: SessionId",
            agent_id AS "agent_id: AgentId",
            agent_scope,
            content_input_bytes AS "content_input_bytes!",
            content_output_bytes AS "content_output_bytes!",
            decision, policy, reason, trace_id, ar_latency_ms,
            metadata AS "metadata!",
            (SELECT COUNT(*) FROM joined)::bigint AS "total_count!"
        FROM joined
        ORDER BY
            (CASE WHEN $11 = 'created_at' AND $12 = 'asc'  THEN created_at END) ASC  NULLS LAST,
            (CASE WHEN $11 = 'created_at' AND $12 = 'desc' THEN created_at END) DESC NULLS LAST,
            (CASE WHEN $11 = 'bytes' AND $12 = 'asc'  THEN (content_input_bytes + content_output_bytes) END) ASC  NULLS LAST,
            (CASE WHEN $11 = 'bytes' AND $12 = 'desc' THEN (content_input_bytes + content_output_bytes) END) DESC NULLS LAST,
            (CASE WHEN $11 = 'latency' AND $12 = 'asc'  THEN ar_latency_ms END) ASC  NULLS LAST,
            (CASE WHEN $11 = 'latency' AND $12 = 'desc' THEN ar_latency_ms END) DESC NULLS LAST
        LIMIT $9 OFFSET $10"#,
        range.from,
        range.to,
        filter.tool_name.as_deref(),
        filter.user_id.as_ref().map(UserId::as_str),
        filter.agent_scope.as_deref(),
        filter.plugin_id.as_deref(),
        filter.decision.as_deref(),
        search_pattern,
        limit,
        offset,
        sort_col,
        sort_dir,
    )
    .fetch_all(pool)
    .await
}

impl From<ToolCallRowWithTotal> for ToolCallRow {
    fn from(r: ToolCallRowWithTotal) -> Self {
        Self {
            id: r.id,
            created_at: r.created_at,
            event_type: r.event_type,
            tool_name: r.tool_name,
            plugin_id: r.plugin_id,
            user_id: r.user_id,
            session_id: r.session_id,
            agent_id: r.agent_id,
            agent_scope: r.agent_scope,
            content_input_bytes: r.content_input_bytes,
            content_output_bytes: r.content_output_bytes,
            decision: r.decision,
            policy: r.policy,
            reason: r.reason,
            trace_id: r.trace_id,
            ar_latency_ms: r.ar_latency_ms,
            metadata: r.metadata,
        }
    }
}
