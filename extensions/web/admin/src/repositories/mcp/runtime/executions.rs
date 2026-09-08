//! Tool-call traffic per MCP server: the rollup, the per-tool split, and the
//! raw call log.
//!
//! `mcp_tool_executions` is the only record of what a server actually did, so
//! every figure the MCP pages show about behaviour comes from here. Failures
//! are counted separately from timeouts because they are different operator
//! problems: a failing tool is a bug, a timing-out one is capacity.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

#[derive(Debug, Clone)]
pub struct McpServerActivity {
    pub server_name: String,
    pub calls: i64,
    pub failures: i64,
    pub timeouts: i64,
    pub distinct_users: i64,
    pub distinct_tools: i64,
    pub avg_ms: Option<f64>,
    pub p95_ms: Option<f64>,
    pub last_call_at: Option<chrono::DateTime<chrono::Utc>>,
    pub prior_calls: i64,
}

// Why: Per-server call volume over a window, with the preceding window of equal
// length beside it so every headline number can carry a delta.
pub async fn list_mcp_server_activity(
    pool: &PgPool,
    window_hours: i64,
) -> Result<Vec<McpServerActivity>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"WITH windowed AS (
             SELECT * FROM mcp_tool_executions
              WHERE started_at >= NOW() - make_interval(hours => $1::int)
           ),
           prior AS (
             SELECT server_name, COUNT(*)::BIGINT AS calls
               FROM mcp_tool_executions
              WHERE started_at >= NOW() - make_interval(hours => $1::int * 2)
                AND started_at <  NOW() - make_interval(hours => $1::int)
              GROUP BY server_name
           )
           SELECT
             w.server_name AS "server_name!",
             COUNT(*)::BIGINT AS "calls!",
             COUNT(*) FILTER (WHERE w.status = 'failed')::BIGINT AS "failures!",
             COUNT(*) FILTER (WHERE w.status = 'timeout')::BIGINT AS "timeouts!",
             COUNT(DISTINCT w.user_id)::BIGINT AS "distinct_users!",
             COUNT(DISTINCT w.tool_name)::BIGINT AS "distinct_tools!",
             AVG(w.execution_time_ms)::FLOAT8 AS "avg_ms?",
             PERCENTILE_CONT(0.95) WITHIN GROUP (
               ORDER BY w.execution_time_ms
             )::FLOAT8 AS "p95_ms?",
             MAX(w.started_at) AS "last_call_at?",
             COALESCE(p.calls, 0)::BIGINT AS "prior_calls!"
           FROM windowed w
           LEFT JOIN prior p ON p.server_name = w.server_name
          GROUP BY w.server_name, p.calls
          ORDER BY COUNT(*) DESC"#,
        i32::try_from(window_hours).unwrap_or(24)
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| McpServerActivity {
            server_name: r.server_name,
            calls: r.calls,
            failures: r.failures,
            timeouts: r.timeouts,
            distinct_users: r.distinct_users,
            distinct_tools: r.distinct_tools,
            avg_ms: r.avg_ms,
            p95_ms: r.p95_ms,
            last_call_at: r.last_call_at,
            prior_calls: r.prior_calls,
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct McpToolStat {
    pub tool_name: String,
    pub calls: i64,
    pub failures: i64,
    pub distinct_users: i64,
    pub avg_ms: Option<f64>,
    pub max_ms: Option<i32>,
    pub last_call_at: Option<chrono::DateTime<chrono::Utc>>,
}

// Why: The tools one server has actually served, busiest first.
pub async fn list_mcp_tool_stats(
    pool: &PgPool,
    server_name: &str,
    window_hours: i64,
    limit: i64,
) -> Result<Vec<McpToolStat>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
             tool_name AS "tool_name!",
             COUNT(*)::BIGINT AS "calls!",
             COUNT(*) FILTER (WHERE status IN ('failed', 'timeout'))::BIGINT AS "failures!",
             COUNT(DISTINCT user_id)::BIGINT AS "distinct_users!",
             AVG(execution_time_ms)::FLOAT8 AS "avg_ms?",
             MAX(execution_time_ms) AS "max_ms?",
             MAX(started_at) AS "last_call_at?"
           FROM mcp_tool_executions
          WHERE server_name = $1
            AND started_at >= NOW() - make_interval(hours => $2::int)
          GROUP BY tool_name
          ORDER BY COUNT(*) DESC
          LIMIT $3"#,
        server_name,
        i32::try_from(window_hours).unwrap_or(24),
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| McpToolStat {
            tool_name: r.tool_name,
            calls: r.calls,
            failures: r.failures,
            distinct_users: r.distinct_users,
            avg_ms: r.avg_ms,
            max_ms: r.max_ms,
            last_call_at: r.last_call_at,
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct McpExecutionRow {
    pub execution_id: String,
    pub tool_name: String,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub execution_time_ms: Option<i32>,
    pub user_id: UserId,
    pub session_id: Option<SessionId>,
    pub trace_id: Option<String>,
    pub error_message: Option<String>,
}

// Why: One page of a server's call log, newest first, with the unpaged total.
pub async fn list_mcp_executions_paged(
    pool: &PgPool,
    server_name: &str,
    limit: i64,
    offset: i64,
) -> Result<(Vec<McpExecutionRow>, i64), sqlx::Error> {
    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*)::BIGINT AS "n!" FROM mcp_tool_executions WHERE server_name = $1"#,
        server_name
    )
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query!(
        r#"SELECT
             mcp_execution_id AS "execution_id!",
             tool_name AS "tool_name!",
             status AS "status!",
             started_at AS "started_at!",
             execution_time_ms AS "execution_time_ms?",
             user_id AS "user_id!: UserId",
             session_id AS "session_id?: SessionId",
             trace_id AS "trace_id?",
             error_message AS "error_message?"
           FROM mcp_tool_executions
          WHERE server_name = $1
          ORDER BY started_at DESC
          LIMIT $2 OFFSET $3"#,
        server_name,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok((
        rows.into_iter()
            .map(|r| McpExecutionRow {
                execution_id: r.execution_id,
                tool_name: r.tool_name,
                status: r.status,
                started_at: r.started_at,
                execution_time_ms: r.execution_time_ms,
                user_id: r.user_id,
                session_id: r.session_id,
                trace_id: r.trace_id,
                error_message: r.error_message,
            })
            .collect(),
        total,
    ))
}
