use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::super::super::types::{HookEventTypeStat, HookSummaryStats, HookTimeSeriesBucket};

pub async fn get_hook_event_breakdown(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<HookEventTypeStat>, sqlx::Error> {
    sqlx::query_as!(
        HookEventTypeStat,
        r#"SELECT
            event_type,
            COALESCE(SUM(event_count), 0)::BIGINT AS "event_count!",
            COALESCE(SUM(error_count), 0)::BIGINT AS "error_count!",
            COALESCE(SUM(content_input_bytes), 0)::BIGINT AS "content_input_bytes!",
            COALESCE(SUM(content_output_bytes), 0)::BIGINT AS "content_output_bytes!"
        FROM plugin_usage_daily
        WHERE user_id = $1
        GROUP BY event_type
        ORDER BY 2 DESC"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn get_hook_timeseries(
    pool: &PgPool,
    user_id: &UserId,
    range: &str,
) -> Result<Vec<HookTimeSeriesBucket>, sqlx::Error> {
    let (interval, bucket_interval, trunc) = match range {
        "24h" => ("24 hours", "1 hour", "hour"),
        "14d" => ("14 days", "1 day", "day"),
        _ => ("7 days", "4 hours", "hour"),
    };

    let sql = format!(
        r"WITH buckets AS (
            SELECT generate_series(
                date_trunc('{trunc}', NOW() - INTERVAL '{interval}'),
                NOW(),
                INTERVAL '{bucket_interval}'
            ) AS bucket
        )
        SELECT
            b.bucket,
            COALESCE(SUM(p.event_count), 0)::BIGINT AS event_count,
            COALESCE(SUM(p.error_count), 0)::BIGINT AS error_count
        FROM buckets b
        LEFT JOIN plugin_usage_daily p
            ON date_trunc('{trunc}', p.date::timestamptz) = b.bucket
            AND p.user_id = $1
        GROUP BY b.bucket
        ORDER BY b.bucket"
    );

    sqlx::query_as::<_, HookTimeSeriesBucket>(&sql)
        .bind(user_id.as_str())
        .fetch_all(pool)
        .await
}

pub async fn get_hook_summary_stats(
    pool: &PgPool,
    user_id: &UserId,
    range: &str,
) -> HookSummaryStats {
    #[derive(sqlx::FromRow)]
    struct StatsRow {
        total_events: i64,
        total_errors: i64,
        content_input_bytes: i64,
        content_output_bytes: i64,
    }

    let interval = match range {
        "24h" => "24 hours",
        "14d" => "14 days",
        _ => "7 days",
    };

    let sql = format!(
        r"SELECT
            COALESCE(SUM(event_count), 0)::BIGINT AS total_events,
            COALESCE(SUM(error_count), 0)::BIGINT AS total_errors,
            COALESCE(SUM(content_input_bytes), 0)::BIGINT AS content_input_bytes,
            COALESCE(SUM(content_output_bytes), 0)::BIGINT AS content_output_bytes
        FROM plugin_usage_daily
        WHERE user_id = $1
            AND date >= (CURRENT_DATE - INTERVAL '{interval}')"
    );

    match sqlx::query_as::<_, StatsRow>(&sql)
        .bind(user_id.as_str())
        .fetch_one(pool)
        .await
    {
        Ok(row) => HookSummaryStats {
            total_events: row.total_events,
            total_errors: row.total_errors,
            content_input_bytes: row.content_input_bytes,
            content_output_bytes: row.content_output_bytes,
        },
        Err(e) => {
            tracing::warn!(error = %e, "Failed to query hook summary stats");
            HookSummaryStats {
                total_events: 0,
                total_errors: 0,
                content_input_bytes: 0,
                content_output_bytes: 0,
            }
        }
    }
}
