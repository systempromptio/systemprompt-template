use sqlx::PgPool;

use super::queries::{FunnelRow, LandingRow, SparkSessionRow, SparkSignupRow};

pub type FunnelSparklineResult = (
    FunnelRow,
    Vec<LandingRow>,
    Vec<SparkSessionRow>,
    Vec<SparkSignupRow>,
);

pub async fn fetch_funnel_and_sparklines(
    pool: &PgPool,
) -> Result<FunnelSparklineResult, sqlx::Error> {
    let funnel = fetch_funnel(pool).await?;
    let landing = fetch_landing(pool).await?;
    let spark_sessions = fetch_spark_sessions(pool).await?;
    let spark_signups = fetch_spark_signups(pool).await?;
    Ok((funnel, landing, spark_sessions, spark_signups))
}

async fn fetch_funnel(pool: &PgPool) -> Result<FunnelRow, sqlx::Error> {
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
    Ok(FunnelRow {
        total_published: funnel.total_published,
        avg_views: funnel.avg_views,
        total_shares: funnel.total_shares,
        total_comments: funnel.total_comments,
    })
}

async fn fetch_landing(pool: &PgPool) -> Result<Vec<LandingRow>, sqlx::Error> {
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
    Ok(landing
        .into_iter()
        .map(|r| LandingRow {
            page_url: r.page_url,
            sessions: r.sessions,
            avg_time_seconds: r.avg_time_seconds,
        })
        .collect())
}

async fn fetch_spark_sessions(pool: &PgPool) -> Result<Vec<SparkSessionRow>, sqlx::Error> {
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
    Ok(spark_sessions
        .into_iter()
        .map(|r| SparkSessionRow {
            day: r.day,
            sessions: r.sessions,
        })
        .collect())
}

async fn fetch_spark_signups(pool: &PgPool) -> Result<Vec<SparkSignupRow>, sqlx::Error> {
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
    Ok(spark_signups
        .into_iter()
        .map(|r| SparkSignupRow {
            day: r.day,
            signups: r.signups,
        })
        .collect())
}
