use sqlx::PgPool;

#[derive(Debug, Clone, Copy)]
pub struct RequestStatsRow {
    pub total: i64,
    pub tool_uses: i64,
    pub errors: i64,
    pub sessions: i64,
}

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
