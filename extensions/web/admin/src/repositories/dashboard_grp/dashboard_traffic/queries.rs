use sqlx::PgPool;

use crate::types::{
    ContentPerformanceRow, RealtimePulse, TrafficCountryBucket, TrafficTopPage,
    TRAFFIC_RANGE_30D, TRAFFIC_RANGE_YESTERDAY,
};

pub async fn fetch_realtime_pulse(pool: &PgPool) -> Result<RealtimePulse, sqlx::Error> {
    let sessions_this_hour: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*)::BIGINT AS "count!"
        FROM user_sessions
        WHERE started_at >= date_trunc('hour', NOW())
          AND NOT is_bot AND NOT is_scanner
          AND NOT COALESCE(is_behavioral_bot, false)
          AND request_count > 0"#
    )
    .fetch_one(pool)
    .await?;

    let page_views_this_hour: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*)::BIGINT AS "count!"
        FROM engagement_events
        WHERE created_at >= date_trunc('hour', NOW())
          AND time_on_page_ms > 0"#
    )
    .fetch_one(pool)
    .await?;

    let unique_visitors_today: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(DISTINCT session_id)::BIGINT AS "count!"
        FROM engagement_events
        WHERE created_at >= CURRENT_DATE"#
    )
    .fetch_one(pool)
    .await?;

    Ok(RealtimePulse {
        sessions_this_hour,
        page_views_this_hour,
        unique_visitors_today,
    })
}

pub async fn fetch_top_pages_today(pool: &PgPool) -> Result<Vec<TrafficTopPage>, sqlx::Error> {
    sqlx::query_as!(
        TrafficTopPage,
        r#"SELECT
            page_url,
            COUNT(*)::BIGINT AS "events!",
            COUNT(DISTINCT session_id)::BIGINT AS "sessions!",
            COALESCE(AVG(LEAST(NULLIF(time_on_page_ms, 0), 600000)), 0)::DOUBLE PRECISION AS "avg_time_ms!"
        FROM engagement_events
        WHERE created_at >= CURRENT_DATE
          AND time_on_page_ms > 0
        GROUP BY page_url
        ORDER BY 2 DESC
        LIMIT 3"#,
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_traffic_country_timeseries(
    pool: &PgPool,
    interval: &str,
    bucket_size: &str,
) -> Result<Vec<TrafficCountryBucket>, sqlx::Error> {
    sqlx::query_as!(
        TrafficCountryBucket,
        r#"WITH top_countries AS (
            SELECT COALESCE(NULLIF(country, ''), 'Unknown') AS c
            FROM user_sessions
            WHERE started_at >= NOW() - $1::text::interval
              AND NOT is_bot AND NOT is_scanner
              AND NOT COALESCE(is_behavioral_bot, false)
              AND request_count > 0
            GROUP BY 1
            ORDER BY COUNT(*) DESC
            LIMIT 10
        )
        SELECT
            date_trunc($2, s.started_at) AS "bucket!",
            CASE WHEN tc.c IS NOT NULL THEN COALESCE(NULLIF(s.country, ''), 'Unknown') ELSE 'Other' END AS "country!",
            COUNT(*)::BIGINT AS "sessions!"
        FROM user_sessions s
        LEFT JOIN top_countries tc ON COALESCE(NULLIF(s.country, ''), 'Unknown') = tc.c
        WHERE s.started_at >= NOW() - $1::text::interval
          AND NOT s.is_bot AND NOT s.is_scanner
          AND NOT COALESCE(s.is_behavioral_bot, false)
          AND s.request_count > 0
        GROUP BY 1, 2
        ORDER BY 1, 3 DESC"#,
        interval,
        bucket_size,
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_content_performance(
    pool: &PgPool,
    content_range: &str,
) -> Result<Vec<ContentPerformanceRow>, sqlx::Error> {
    match content_range {
        "7d" | "30d" => fetch_content_performance_precomputed(pool, content_range).await,
        _ => fetch_content_performance_live(pool, content_range).await,
    }
}

async fn fetch_content_performance_precomputed(
    pool: &PgPool,
    content_range: &str,
) -> Result<Vec<ContentPerformanceRow>, sqlx::Error> {
    let use_30d = content_range == TRAFFIC_RANGE_30D;
    let rows = sqlx::query_as!(
        ContentPerfPrecomputed,
        r#"SELECT
            COALESCE(mc.title, cpm.content_id) AS "title!",
            CASE WHEN $1 THEN cpm.views_last_30_days::BIGINT ELSE cpm.views_last_7_days::BIGINT END AS "views!",
            cpm.trend_direction AS "trend?",
            cpm.avg_time_on_page_seconds AS "avg_time_seconds!"
        FROM content_performance_metrics cpm
        JOIN markdown_content mc ON mc.id = cpm.content_id
        WHERE CASE WHEN $1 THEN cpm.views_last_30_days > 0 ELSE cpm.views_last_7_days > 0 END
        ORDER BY CASE WHEN $1 THEN cpm.views_last_30_days ELSE cpm.views_last_7_days END DESC
        LIMIT 10"#,
        use_30d,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ContentPerformanceRow {
            title: r.title,
            views: r.views,
            trend: r.trend,
            avg_time_seconds: r.avg_time_seconds,
        })
        .collect())
}

async fn fetch_content_performance_live(
    pool: &PgPool,
    content_range: &str,
) -> Result<Vec<ContentPerformanceRow>, sqlx::Error> {
    let interval = match content_range {
        "1h" => "1 hour",
        "yesterday" => "48 hours",
        _ => "24 hours",
    };
    let is_yesterday = content_range == TRAFFIC_RANGE_YESTERDAY;

    let rows = sqlx::query_as!(
        ContentPerfLive,
        r#"SELECT
            COALESCE(mc.title, ee.page_url) AS "title!",
            COUNT(*)::BIGINT AS "views!",
            COALESCE(AVG(LEAST(NULLIF(ee.time_on_page_ms, 0), 600000)) / 1000.0, 0)::FLOAT8 AS "avg_time_seconds!"
        FROM engagement_events ee
        LEFT JOIN markdown_content mc ON ee.page_url = '/' || mc.source_id || '/' || mc.slug
        WHERE ee.time_on_page_ms > 0
          AND CASE
            WHEN $2 THEN ee.created_at >= (CURRENT_DATE - INTERVAL '1 day') AND ee.created_at < CURRENT_DATE
            ELSE ee.created_at >= NOW() - $1::text::interval
        END
        GROUP BY COALESCE(mc.title, ee.page_url)
        HAVING COUNT(*) > 0
        ORDER BY COUNT(*) DESC
        LIMIT 10"#,
        interval,
        is_yesterday,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ContentPerformanceRow {
            title: r.title,
            views: r.views,
            trend: None,
            avg_time_seconds: r.avg_time_seconds,
        })
        .collect())
}

#[derive(Debug, sqlx::FromRow)]
struct ContentPerfPrecomputed {
    title: String,
    views: i64,
    trend: Option<String>,
    avg_time_seconds: f64,
}

#[derive(Debug, sqlx::FromRow)]
struct ContentPerfLive {
    title: String,
    views: i64,
    avg_time_seconds: f64,
}
