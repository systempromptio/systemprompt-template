use sqlx::PgPool;

use crate::types::{ActivityStats, TimeSeriesBucket};

pub async fn get_activity_stats(pool: &PgPool) -> Result<ActivityStats, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
            COALESCE(COUNT(*) FILTER (WHERE created_at >= CURRENT_DATE), 0)::BIGINT AS "events_today!",
            COALESCE(COUNT(*) FILTER (WHERE created_at >= DATE_TRUNC('week', CURRENT_DATE)), 0)::BIGINT AS "events_this_week!",
            COALESCE(COUNT(*) FILTER (WHERE category = 'marketplace_edit'), 0)::BIGINT AS "total_edits!",
            COALESCE(COUNT(*) FILTER (WHERE category = 'login'), 0)::BIGINT AS "total_logins!"
        FROM user_activity"#,
    )
    .fetch_one(pool)
    .await?;

    let mcp_row = sqlx::query!(
        r#"SELECT
            COUNT(*)::BIGINT AS "mcp_tool_calls!",
            COALESCE(COUNT(*) FILTER (WHERE status = 'failed'), 0)::BIGINT AS "mcp_errors!"
        FROM mcp_tool_executions"#,
    )
    .fetch_one(pool)
    .await;

    let (mcp_tool_calls, mcp_errors) = match mcp_row {
        Ok(r) => (r.mcp_tool_calls, r.mcp_errors),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to query mcp_tool_executions for dashboard stats");
            (0, 0)
        },
    };

    Ok(ActivityStats {
        events_today: row.events_today,
        events_this_week: row.events_this_week,
        total_edits: row.total_edits,
        mcp_tool_calls,
        mcp_errors,
        total_logins: row.total_logins,
    })
}

pub async fn fetch_usage_timeseries(
    pool: &PgPool,
    interval: &str,
    bucket_interval: &str,
) -> Result<Vec<TimeSeriesBucket>, sqlx::Error> {
    let trunc = if bucket_interval.contains("hour") {
        "hour"
    } else {
        "day"
    };
    sqlx::query_as!(
        TimeSeriesBucket,
        r#"WITH buckets AS (
            SELECT generate_series(
                date_trunc($1, NOW() - $2::text::interval),
                NOW(),
                $3::text::interval
            ) AS bucket
        ),
        activity AS (
            SELECT
                date_trunc($1, a.created_at) AS bucket,
                0::BIGINT AS mcp_calls,
                COUNT(*) FILTER (WHERE a.category = 'marketplace_edit')::BIGINT AS edits,
                COUNT(DISTINCT a.user_id)::BIGINT AS active_users,
                COUNT(*) FILTER (WHERE a.category = 'login')::BIGINT AS logins,
                0::BIGINT AS mcp_errors
            FROM user_activity a
            WHERE a.created_at >= NOW() - $2::text::interval
            GROUP BY 1
        ),
        mcp AS (
            SELECT
                date_trunc($1, m.created_at) AS bucket,
                COUNT(*)::BIGINT AS mcp_calls,
                0::BIGINT AS edits,
                0::BIGINT AS active_users,
                0::BIGINT AS logins,
                COUNT(*) FILTER (WHERE m.status = 'failed')::BIGINT AS mcp_errors
            FROM mcp_tool_executions m
            WHERE m.created_at >= NOW() - $2::text::interval
            GROUP BY 1
        )
        SELECT
            b.bucket AS "bucket!",
            COALESCE(SUM(d.mcp_calls), 0)::BIGINT AS "tool_uses!",
            COALESCE(SUM(d.edits), 0)::BIGINT AS "prompts!",
            COALESCE(SUM(d.active_users), 0)::BIGINT AS "active_users!",
            COALESCE(SUM(d.logins), 0)::BIGINT AS "sessions!",
            COALESCE(SUM(d.mcp_errors), 0)::BIGINT AS "errors!"
        FROM buckets b
        LEFT JOIN (
            SELECT * FROM activity UNION ALL SELECT * FROM mcp
        ) d ON d.bucket = b.bucket
        GROUP BY b.bucket
        ORDER BY b.bucket"#,
        trunc,
        interval,
        bucket_interval,
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_active_users_24h(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT COALESCE(COUNT(DISTINCT combined.user_id), 0)::BIGINT as "count!"
        FROM (
            SELECT user_id FROM user_activity WHERE created_at >= NOW() - INTERVAL '24 hours'
            UNION
            SELECT user_id FROM mcp_tool_executions WHERE created_at >= NOW() - INTERVAL '24 hours' AND user_id IS NOT NULL
        ) combined
        JOIN users u ON u.id = combined.user_id
        WHERE NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'"#,
    )
    .fetch_one(pool)
    .await
}
