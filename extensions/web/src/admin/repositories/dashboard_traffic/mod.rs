mod kpis;
mod queries;

pub use queries::{
    fetch_content_performance, fetch_realtime_pulse, fetch_top_pages_today,
    fetch_traffic_country_timeseries,
};

use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::{
    TopPageDailyBucket, TrafficData, TrafficDevice, TrafficGeo, TrafficSource, TrafficTimeBucket,
    TrafficTopPage,
};

fn range_params(range: &str) -> (&str, &str, &str) {
    match range {
        "7d" => ("7 days", "14 days", "day"),
        "30d" => ("30 days", "60 days", "day"),
        _ => ("24 hours", "48 hours", "hour"),
    }
}

pub async fn fetch_traffic_data(
    pool: &Arc<PgPool>,
    range: &str,
) -> Result<TrafficData, sqlx::Error> {
    let (interval, prev_interval, bucket) = range_params(range);

    let (kpis, timeseries, sources, geo, devices, top_pages, country_timeseries, top_pages_daily) =
        tokio::try_join!(
            kpis::fetch_traffic_kpis(pool, interval, prev_interval),
            fetch_traffic_timeseries(pool, interval, bucket),
            fetch_traffic_sources(pool, interval),
            fetch_traffic_geo(pool, interval),
            fetch_traffic_devices(pool, interval),
            fetch_traffic_top_pages(pool, interval),
            fetch_traffic_country_timeseries(pool, interval, bucket),
            fetch_top_pages_daily(pool),
        )?;

    Ok(TrafficData {
        kpis,
        timeseries,
        sources,
        geo,
        devices,
        top_pages,
        country_timeseries,
        top_pages_daily,
    })
}

async fn fetch_traffic_timeseries(
    pool: &Arc<PgPool>,
    interval: &str,
    bucket_size: &str,
) -> Result<Vec<TrafficTimeBucket>, sqlx::Error> {
    sqlx::query_as!(
        TrafficTimeBucket,
        r#"WITH session_buckets AS (
            SELECT
                date_trunc($1, started_at) as bucket,
                COUNT(*) FILTER (
                    WHERE NOT is_bot AND NOT is_scanner
                    AND NOT COALESCE(is_behavioral_bot, false)
                    AND request_count > 0
                )::BIGINT as sessions
            FROM user_sessions
            WHERE started_at >= NOW() - $2::text::interval
            GROUP BY 1
        ),
        event_buckets AS (
            SELECT
                date_trunc($1, created_at) as bucket,
                COUNT(*)::BIGINT as page_views
            FROM engagement_events
            WHERE created_at >= NOW() - $2::text::interval
              AND time_on_page_ms > 0
            GROUP BY 1
        )
        SELECT
            COALESCE(s.bucket, e.bucket) AS "bucket!",
            COALESCE(s.sessions, 0)::BIGINT AS "sessions!",
            COALESCE(e.page_views, 0)::BIGINT AS "page_views!"
        FROM session_buckets s
        FULL OUTER JOIN event_buckets e ON s.bucket = e.bucket
        ORDER BY 1"#,
        bucket_size,
        interval,
    )
    .fetch_all(pool.as_ref())
    .await
}

async fn fetch_traffic_sources(
    pool: &Arc<PgPool>,
    interval: &str,
) -> Result<Vec<TrafficSource>, sqlx::Error> {
    sqlx::query_as!(
        TrafficSource,
        r#"SELECT
            CASE
                WHEN referrer_source IS NULL OR referrer_source = '' THEN 'Direct'
                WHEN referrer_source IN ('systemprompt.io', 'www.systemprompt.io') THEN 'Direct'
                ELSE referrer_source
            END AS "source!",
            COUNT(*)::BIGINT AS "sessions!"
        FROM user_sessions
        WHERE started_at >= NOW() - $1::text::interval
          AND NOT is_bot AND NOT is_scanner
          AND NOT COALESCE(is_behavioral_bot, false)
          AND request_count > 0
        GROUP BY 1
        ORDER BY 2 DESC
        LIMIT 10"#,
        interval,
    )
    .fetch_all(pool.as_ref())
    .await
}

async fn fetch_traffic_geo(
    pool: &Arc<PgPool>,
    interval: &str,
) -> Result<Vec<TrafficGeo>, sqlx::Error> {
    sqlx::query_as!(
        TrafficGeo,
        r#"SELECT
            COALESCE(NULLIF(country, ''), 'Unknown') AS "country!",
            COUNT(*)::BIGINT AS "sessions!"
        FROM user_sessions
        WHERE started_at >= NOW() - $1::text::interval
          AND NOT is_bot AND NOT is_scanner
          AND NOT COALESCE(is_behavioral_bot, false)
          AND request_count > 0
        GROUP BY 1
        ORDER BY 2 DESC
        LIMIT 10"#,
        interval,
    )
    .fetch_all(pool.as_ref())
    .await
}

async fn fetch_traffic_devices(
    pool: &Arc<PgPool>,
    interval: &str,
) -> Result<Vec<TrafficDevice>, sqlx::Error> {
    sqlx::query_as!(
        TrafficDevice,
        r#"SELECT
            COALESCE(NULLIF(device_type, ''), 'Unknown') AS "device!",
            COUNT(*)::BIGINT AS "sessions!"
        FROM user_sessions
        WHERE started_at >= NOW() - $1::text::interval
          AND NOT is_bot AND NOT is_scanner
          AND NOT COALESCE(is_behavioral_bot, false)
          AND request_count > 0
        GROUP BY 1
        ORDER BY 2 DESC
        LIMIT 10"#,
        interval,
    )
    .fetch_all(pool.as_ref())
    .await
}

async fn fetch_traffic_top_pages(
    pool: &Arc<PgPool>,
    interval: &str,
) -> Result<Vec<TrafficTopPage>, sqlx::Error> {
    sqlx::query_as!(
        TrafficTopPage,
        r#"SELECT
            page_url,
            COUNT(*)::BIGINT AS "events!",
            COUNT(DISTINCT session_id)::BIGINT AS "sessions!",
            COALESCE(AVG(LEAST(time_on_page_ms, 600000)), 0)::DOUBLE PRECISION AS "avg_time_ms!"
        FROM engagement_events
        WHERE created_at >= NOW() - $1::text::interval
          AND time_on_page_ms > 0
        GROUP BY page_url
        ORDER BY 2 DESC
        LIMIT 15"#,
        interval,
    )
    .fetch_all(pool.as_ref())
    .await
}

async fn fetch_top_pages_daily(pool: &Arc<PgPool>) -> Result<Vec<TopPageDailyBucket>, sqlx::Error> {
    sqlx::query_as!(
        TopPageDailyBucket,
        r#"WITH top_pages AS (
            SELECT page_url
            FROM engagement_events
            WHERE created_at >= NOW() - INTERVAL '31 days'
              AND time_on_page_ms > 0
            GROUP BY page_url
            ORDER BY COUNT(*) DESC
            LIMIT 10
        )
        SELECT
            tp.page_url AS "page_url!",
            e.created_at::date AS "day!",
            COUNT(*)::BIGINT AS "views!",
            COUNT(DISTINCT e.session_id)::BIGINT AS "sessions!",
            COALESCE(AVG(LEAST(e.time_on_page_ms, 600000)), 0)::DOUBLE PRECISION AS "avg_time_ms!"
        FROM top_pages tp
        JOIN engagement_events e ON e.page_url = tp.page_url
        WHERE e.created_at >= NOW() - INTERVAL '31 days'
          AND e.time_on_page_ms > 0
        GROUP BY tp.page_url, e.created_at::date
        ORDER BY tp.page_url, e.created_at::date"#,
    )
    .fetch_all(pool.as_ref())
    .await
}
