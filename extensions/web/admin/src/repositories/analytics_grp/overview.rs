use sqlx::PgPool;

#[derive(Debug, Clone, Copy)]
pub struct OverviewRow {
    pub total_events: i64,
    pub total_sessions: i64,
    pub tool_uses: i64,
    pub errors: i64,
}

pub async fn get_overview(pool: &PgPool) -> Result<OverviewRow, sqlx::Error> {
    sqlx::query_as!(
        OverviewRow,
        r#"SELECT
            COUNT(*)::bigint AS "total_events!",
            COUNT(DISTINCT session_id)::bigint AS "total_sessions!",
            COUNT(*) FILTER (WHERE event_type LIKE '%ToolUse%')::bigint AS "tool_uses!",
            COUNT(*) FILTER (WHERE event_type LIKE '%Failure%' OR event_type LIKE '%Error%')::bigint AS "errors!"
          FROM plugin_usage_events
          WHERE created_at >= NOW() - INTERVAL '24 hours'"#,
    )
    .fetch_one(pool)
    .await
}
