use sqlx::PgPool;

use super::data::{
    ContentBreakdownResult, DeviceRow, FunnelRow, GeoRow, LandingRow, SeoRow, SourceRow,
    SparkSessionRow, SparkSignupRow, TopContentRow,
};

pub(super) async fn fetch_content_and_breakdown_data(
    pool: &PgPool,
) -> Result<ContentBreakdownResult, Box<dyn std::error::Error + Send + Sync>> {
    let top_content = fetch_top_content(pool).await?;
    let seo = fetch_seo_metrics(pool).await?;
    let geo = fetch_geo_breakdown(pool).await?;
    let devices = fetch_device_breakdown(pool).await?;
    let sources = fetch_source_breakdown(pool).await?;

    Ok((top_content, seo, geo, devices, sources))
}

async fn fetch_top_content(
    pool: &PgPool,
) -> Result<Vec<TopContentRow>, Box<dyn std::error::Error + Send + Sync>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            mc.title as "title!",
            mc.slug as "slug!",
            cpm.views_last_7_days::BIGINT as "views_7d!",
            cpm.views_last_30_days::BIGINT as "views_30d!",
            cpm.unique_visitors::BIGINT as "unique_visitors!",
            cpm.avg_time_on_page_seconds::FLOAT8 as "avg_time_seconds!",
            COALESCE(cpm.trend_direction, 'stable') as "trend!",
            cpm.search_impressions::BIGINT as "search_impressions!",
            cpm.search_clicks::BIGINT as "search_clicks!"
        FROM content_performance_metrics cpm
        JOIN markdown_content mc ON mc.id = cpm.content_id
        ORDER BY cpm.views_last_7_days DESC
        LIMIT 15
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| TopContentRow {
            title: r.title,
            slug: r.slug,
            views_7d: r.views_7d,
            views_30d: r.views_30d,
            unique_visitors: r.unique_visitors,
            avg_time_seconds: r.avg_time_seconds,
            trend: r.trend,
            search_impressions: r.search_impressions,
            search_clicks: r.search_clicks,
        })
        .collect())
}

async fn fetch_seo_metrics(
    pool: &PgPool,
) -> Result<SeoRow, Box<dyn std::error::Error + Send + Sync>> {
    let row = sqlx::query!(
        r#"
        SELECT
            COALESCE(SUM(search_impressions), 0)::BIGINT as "total_impressions!",
            COALESCE(SUM(search_clicks), 0)::BIGINT as "total_clicks!",
            COUNT(*)::BIGINT as "total_indexed!",
            COALESCE(AVG(avg_search_position), 0.0)::FLOAT8 as "avg_position!"
        FROM content_performance_metrics
        WHERE search_impressions > 0 OR search_clicks > 0
        "#,
    )
    .fetch_one(pool)
    .await?;

    Ok(SeoRow {
        total_impressions: row.total_impressions,
        total_clicks: row.total_clicks,
        total_indexed: row.total_indexed,
        avg_position: row.avg_position,
    })
}

async fn fetch_geo_breakdown(
    pool: &PgPool,
) -> Result<Vec<GeoRow>, Box<dyn std::error::Error + Send + Sync>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            COALESCE(NULLIF(country, ''), 'Unknown') as "country!",
            COUNT(*)::BIGINT as "sessions!"
        FROM user_sessions
        WHERE started_at >= NOW() - INTERVAL '24 hours'
          AND NOT is_bot AND NOT is_scanner AND NOT COALESCE(is_behavioral_bot, false) AND request_count > 0
        GROUP BY 1 ORDER BY 2 DESC LIMIT 10
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| GeoRow {
            country: r.country,
            sessions: r.sessions,
        })
        .collect())
}

async fn fetch_device_breakdown(
    pool: &PgPool,
) -> Result<Vec<DeviceRow>, Box<dyn std::error::Error + Send + Sync>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            COALESCE(NULLIF(device_type, ''), 'Unknown') as "device!",
            COUNT(*)::BIGINT as "sessions!"
        FROM user_sessions
        WHERE started_at >= NOW() - INTERVAL '24 hours'
          AND NOT is_bot AND NOT is_scanner AND NOT COALESCE(is_behavioral_bot, false) AND request_count > 0
        GROUP BY 1 ORDER BY 2 DESC LIMIT 10
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| DeviceRow {
            device: r.device,
            sessions: r.sessions,
        })
        .collect())
}

async fn fetch_source_breakdown(
    pool: &PgPool,
) -> Result<Vec<SourceRow>, Box<dyn std::error::Error + Send + Sync>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            COALESCE(NULLIF(referrer_source, ''), 'Direct') as "source!",
            COUNT(*)::BIGINT as "sessions!"
        FROM user_sessions
        WHERE started_at >= NOW() - INTERVAL '24 hours'
          AND NOT is_bot AND NOT is_scanner AND NOT COALESCE(is_behavioral_bot, false) AND request_count > 0
        GROUP BY 1 ORDER BY 2 DESC LIMIT 10
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| SourceRow {
            source: r.source,
            sessions: r.sessions,
        })
        .collect())
}

pub(super) type FunnelSparklineResult = (
    FunnelRow,
    Vec<LandingRow>,
    Vec<SparkSessionRow>,
    Vec<SparkSignupRow>,
);

pub(super) async fn fetch_funnel_and_sparklines(
    pool: &PgPool,
) -> Result<FunnelSparklineResult, Box<dyn std::error::Error + Send + Sync>> {
    let funnel = sqlx::query!(
        r#"
        SELECT
            COUNT(*)::BIGINT as "total_published!",
            CASE WHEN COUNT(*) > 0
                 THEN COALESCE(SUM(total_views), 0)::FLOAT8 / COUNT(*)::FLOAT8
                 ELSE 0.0
            END as "avg_views!",
            COALESCE(SUM(shares_total), 0)::BIGINT as "total_shares!",
            COALESCE(SUM(comments_count), 0)::BIGINT as "total_comments!"
        FROM content_performance_metrics
        "#,
    )
    .fetch_one(pool)
    .await?;

    let landing = sqlx::query!(
        r#"
        SELECT
            COALESCE(landing_page, entry_url, 'Unknown') as "page_url!",
            COUNT(*)::BIGINT as "sessions!",
            AVG(EXTRACT(EPOCH FROM (last_activity_at - started_at)))::FLOAT8 as "avg_time_seconds!"
        FROM user_sessions
        WHERE started_at >= NOW() - INTERVAL '24 hours'
          AND NOT is_bot AND NOT is_scanner AND NOT COALESCE(is_behavioral_bot, false) AND request_count > 0
          AND (landing_page IS NOT NULL OR entry_url IS NOT NULL)
        GROUP BY 1 ORDER BY 2 DESC LIMIT 10
        "#,
    )
    .fetch_all(pool)
    .await?;

    let spark_sessions = sqlx::query!(
        r#"
        SELECT (started_at::date) as "day!: chrono::NaiveDate", COUNT(*)::BIGINT as "sessions!"
        FROM user_sessions
        WHERE started_at >= NOW() - INTERVAL '7 days'
          AND NOT is_bot AND NOT is_scanner AND NOT COALESCE(is_behavioral_bot, false) AND request_count > 0
        GROUP BY 1 ORDER BY 1
        "#,
    )
    .fetch_all(pool)
    .await?;

    let spark_signups = sqlx::query!(
        r#"
        SELECT (remote_created_at::date) as "day!: chrono::NaiveDate", COUNT(*)::BIGINT as "signups!"
        FROM tenant_activity
        WHERE remote_created_at >= NOW() - INTERVAL '7 days' AND event_type = 'user_created'
        GROUP BY 1 ORDER BY 1
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok((
        FunnelRow {
            total_published: funnel.total_published,
            avg_views: funnel.avg_views,
            total_shares: funnel.total_shares,
            total_comments: funnel.total_comments,
        },
        landing
            .into_iter()
            .map(|r| LandingRow {
                page_url: r.page_url,
                sessions: r.sessions,
                avg_time_seconds: r.avg_time_seconds,
            })
            .collect(),
        spark_sessions
            .into_iter()
            .map(|r| SparkSessionRow {
                day: r.day,
                sessions: r.sessions,
            })
            .collect(),
        spark_signups
            .into_iter()
            .map(|r| SparkSignupRow {
                day: r.day,
                signups: r.signups,
            })
            .collect(),
    ))
}

pub(super) fn build_sparkline_arrays(
    today: chrono::NaiveDate,
    spark_sessions: &[SparkSessionRow],
    spark_signups: &[SparkSignupRow],
) -> (Vec<i64>, Vec<i64>, Vec<String>) {
    let mut spark_sess_arr = Vec::new();
    let mut spark_signup_arr = Vec::new();
    let mut spark_labels = Vec::new();
    for i in (0..7).rev() {
        let day = today - chrono::Duration::days(i);
        spark_labels.push(day.format("%b %d").to_string());
        spark_sess_arr.push(
            spark_sessions
                .iter()
                .find(|r| r.day == day)
                .map_or(0, |r| r.sessions),
        );
        spark_signup_arr.push(
            spark_signups
                .iter()
                .find(|r| r.day == day)
                .map_or(0, |r| r.signups),
        );
    }
    (spark_sess_arr, spark_signup_arr, spark_labels)
}
