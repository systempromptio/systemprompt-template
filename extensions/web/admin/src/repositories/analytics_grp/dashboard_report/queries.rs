use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct TopContentRow {
    pub title: String,
    pub slug: String,
    pub views_7d: i64,
    pub views_30d: i64,
    pub unique_visitors: i64,
    pub avg_time_seconds: f64,
    pub trend: String,
    pub search_impressions: i64,
    pub search_clicks: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct SeoRow {
    pub total_impressions: i64,
    pub total_clicks: i64,
    pub total_indexed: i64,
    pub avg_position: f64,
}

#[derive(Debug, Clone)]
pub struct GeoRow {
    pub country: String,
    pub sessions: i64,
}

#[derive(Debug, Clone)]
pub struct DeviceRow {
    pub device: String,
    pub sessions: i64,
}

#[derive(Debug, Clone)]
pub struct SourceRow {
    pub source: String,
    pub sessions: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct FunnelRow {
    pub total_published: i64,
    pub avg_views: f64,
    pub total_shares: i64,
    pub total_comments: i64,
}

#[derive(Debug, Clone)]
pub struct LandingRow {
    pub page_url: String,
    pub sessions: i64,
    pub avg_time_seconds: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct SparkSessionRow {
    pub day: chrono::NaiveDate,
    pub sessions: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct SparkSignupRow {
    pub day: chrono::NaiveDate,
    pub signups: i64,
}

pub type ContentBreakdownResult = (
    Vec<TopContentRow>,
    SeoRow,
    Vec<GeoRow>,
    Vec<DeviceRow>,
    Vec<SourceRow>,
);

pub async fn fetch_content_and_breakdown_data(
    pool: &PgPool,
) -> Result<ContentBreakdownResult, sqlx::Error> {
    let top_content = fetch_top_content(pool).await?;
    let seo = fetch_seo_metrics(pool).await?;
    let geo = fetch_geo_breakdown(pool).await?;
    let devices = fetch_device_breakdown(pool).await?;
    let sources = fetch_source_breakdown(pool).await?;

    Ok((top_content, seo, geo, devices, sources))
}

async fn fetch_top_content(pool: &PgPool) -> Result<Vec<TopContentRow>, sqlx::Error> {
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

async fn fetch_seo_metrics(pool: &PgPool) -> Result<SeoRow, sqlx::Error> {
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

async fn fetch_geo_breakdown(pool: &PgPool) -> Result<Vec<GeoRow>, sqlx::Error> {
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

async fn fetch_device_breakdown(pool: &PgPool) -> Result<Vec<DeviceRow>, sqlx::Error> {
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

async fn fetch_source_breakdown(pool: &PgPool) -> Result<Vec<SourceRow>, sqlx::Error> {
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
