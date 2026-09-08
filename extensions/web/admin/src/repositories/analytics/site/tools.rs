//! Per-tool MCP execution statistics for the Tools tab.
//!
//! `mcp_tool_executions` is the measured plane: every row is one real tool
//! call with its own duration and status, so nothing here is attributed or
//! inferred. `execution_time_ms` is NULL while a call is still pending, which
//! is why the percentile ignores NULLs and the page reports how many rows had
//! no duration rather than treating them as zero.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

use super::SiteScope;

#[derive(Debug, Clone)]
pub struct ToolStatsRow {
    pub server_name: String,
    pub tool_name: String,
    pub executions: i64,
    pub succeeded: i64,
    pub failed: i64,
    pub pending: i64,
    pub p50_ms: Option<f64>,
    pub p95_ms: Option<f64>,
    pub distinct_users: i64,
}

pub async fn list_tool_stats(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
    limit: i64,
    offset: i64,
) -> Result<(Vec<ToolStatsRow>, i64), sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            e.server_name AS "server_name!",
            e.tool_name AS "tool_name!",
            COUNT(*)::BIGINT AS "executions!",
            COUNT(*) FILTER (WHERE e.status = 'success')::BIGINT AS "succeeded!",
            COUNT(*) FILTER (WHERE e.status IN ('failed', 'timeout'))::BIGINT AS "failed!",
            COUNT(*) FILTER (WHERE e.status = 'pending')::BIGINT AS "pending!",
            percentile_cont(0.5) WITHIN GROUP (ORDER BY e.execution_time_ms) AS p50,
            percentile_cont(0.95) WITHIN GROUP (ORDER BY e.execution_time_ms) AS p95,
            COUNT(DISTINCT e.user_id)::BIGINT AS "distinct_users!",
            COUNT(*) OVER ()::BIGINT AS "total!"
        FROM mcp_tool_executions e
        WHERE e.started_at >= $1 AND e.started_at < $2
          AND ($3::TEXT[] IS NULL OR e.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR e.user_id = $4)
        GROUP BY e.server_name, e.tool_name
        ORDER BY COUNT(*) DESC, e.tool_name
        LIMIT $5 OFFSET $6
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = rows.first().map_or(0, |r| r.total);
    Ok((
        rows.into_iter()
            .map(|r| ToolStatsRow {
                server_name: r.server_name,
                tool_name: r.tool_name,
                executions: r.executions,
                succeeded: r.succeeded,
                failed: r.failed,
                pending: r.pending,
                p50_ms: r.p50,
                p95_ms: r.p95,
                distinct_users: r.distinct_users,
            })
            .collect(),
        total,
    ))
}

/// Server-level totals, so the tab can lead with which servers carry the
/// traffic before the per-tool table asks the reader to scan.
#[derive(Debug, Clone)]
pub struct ToolServerRow {
    pub server_name: String,
    pub executions: i64,
    pub succeeded: i64,
    pub tools: i64,
    pub distinct_users: i64,
}

pub async fn list_tool_servers(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<Vec<ToolServerRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            e.server_name AS "server_name!",
            COUNT(*)::BIGINT AS "executions!",
            COUNT(*) FILTER (WHERE e.status = 'success')::BIGINT AS "succeeded!",
            COUNT(DISTINCT e.tool_name)::BIGINT AS "tools!",
            COUNT(DISTINCT e.user_id)::BIGINT AS "distinct_users!"
        FROM mcp_tool_executions e
        WHERE e.started_at >= $1 AND e.started_at < $2
          AND ($3::TEXT[] IS NULL OR e.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR e.user_id = $4)
        GROUP BY e.server_name
        ORDER BY COUNT(*) DESC
        LIMIT 25
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ToolServerRow {
            server_name: r.server_name,
            executions: r.executions,
            succeeded: r.succeeded,
            tools: r.tools,
            distinct_users: r.distinct_users,
        })
        .collect())
}
