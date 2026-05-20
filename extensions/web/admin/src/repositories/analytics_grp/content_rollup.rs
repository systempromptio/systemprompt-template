use sqlx::PgPool;

#[derive(Debug)]
pub struct ContentRollupRow {
    pub content_id: String,
    pub total_views: i64,
    pub unique_visitors: i64,
    pub avg_time_seconds: f64,
    pub views_7d: i64,
    pub views_30d: i64,
}

pub async fn aggregate_engagement_stats(
    pool: &PgPool,
) -> Result<Vec<ContentRollupRow>, sqlx::Error> {
    sqlx::query_as!(
        ContentRollupRow,
        r#"
        SELECT
            mc.id as "content_id!",
            COUNT(*) FILTER (WHERE ee.time_on_page_ms > 0)::BIGINT as "total_views!",
            COUNT(DISTINCT ee.session_id)::BIGINT as "unique_visitors!",
            COALESCE(AVG(ee.time_on_page_ms)::DOUBLE PRECISION / 1000.0, 0) as "avg_time_seconds!",
            COUNT(*) FILTER (
                WHERE ee.time_on_page_ms > 0
                AND ee.created_at >= NOW() - INTERVAL '7 days'
            )::BIGINT as "views_7d!",
            COUNT(*) FILTER (
                WHERE ee.time_on_page_ms > 0
                AND ee.created_at >= NOW() - INTERVAL '30 days'
            )::BIGINT as "views_30d!"
        FROM engagement_events ee
        JOIN markdown_content mc ON (
            (ee.page_url LIKE '/blog/%' AND mc.slug = SUBSTRING(ee.page_url FROM 7) AND mc.source_id = 'blog')
            OR (ee.page_url LIKE '/documentation/%' AND mc.slug = SUBSTRING(ee.page_url FROM 16) AND mc.source_id = 'documentation')
            OR (ee.page_url LIKE '/playbooks/%' AND mc.slug = SUBSTRING(ee.page_url FROM 12) AND mc.source_id = 'playbooks')
            OR (ee.page_url LIKE '/legal/%' AND mc.slug = SUBSTRING(ee.page_url FROM 8) AND mc.source_id = 'legal')
        )
        GROUP BY mc.id
        HAVING COUNT(*) > 0
        "#
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug)]
pub struct UpsertMetricsParams<'a> {
    pub id: &'a str,
    pub content_id: &'a str,
    pub total_views: i32,
    pub unique_visitors: i32,
    pub avg_time_seconds: f64,
    pub views_7d: i32,
    pub views_30d: i32,
    pub trend_direction: &'a str,
}

pub async fn upsert_metrics(
    pool: &PgPool,
    params: &UpsertMetricsParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO content_performance_metrics (
            id, content_id, total_views, unique_visitors,
            avg_time_on_page_seconds, views_last_7_days, views_last_30_days,
            trend_direction, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
        ON CONFLICT (content_id) DO UPDATE SET
            total_views = EXCLUDED.total_views,
            unique_visitors = EXCLUDED.unique_visitors,
            avg_time_on_page_seconds = EXCLUDED.avg_time_on_page_seconds,
            views_last_7_days = EXCLUDED.views_last_7_days,
            views_last_30_days = EXCLUDED.views_last_30_days,
            trend_direction = EXCLUDED.trend_direction,
            updated_at = NOW()
        "#,
        params.id,
        params.content_id,
        params.total_views,
        params.unique_visitors,
        params.avg_time_seconds,
        params.views_7d,
        params.views_30d,
        params.trend_direction,
    )
    .execute(pool)
    .await?;
    Ok(())
}
