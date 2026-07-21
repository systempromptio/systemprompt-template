//! Per-tool usage rollups.

use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct ToolRow {
    pub tool_name: String,
    pub calls: i64,
    pub errors: i64,
    pub sessions: i64,
}

pub async fn list_tools(pool: &PgPool) -> Result<Vec<ToolRow>, sqlx::Error> {
    sqlx::query_as!(
        ToolRow,
        r#"SELECT
            tool_name AS "tool_name!",
            COUNT(*)::bigint AS "calls!",
            COUNT(*) FILTER (WHERE event_type LIKE '%Failure%' OR event_type LIKE '%Error%')::bigint AS "errors!",
            COUNT(DISTINCT session_id)::bigint AS "sessions!"
          FROM plugin_usage_events
          WHERE tool_name IS NOT NULL
            AND created_at >= NOW() - INTERVAL '7 days'
          GROUP BY tool_name
          ORDER BY COUNT(*) DESC
          LIMIT 50"#,
    )
    .fetch_all(pool)
    .await
}
