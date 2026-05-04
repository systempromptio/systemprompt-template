use sqlx::PgPool;

#[derive(Debug, Clone, Copy)]
pub struct SessionsRow {
    pub today: i64,
    pub yesterday: i64,
    pub avg_7d: f64,
    pub avg_14d: f64,
    pub avg_30d: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct PageViewsRow {
    pub today: i64,
    pub yesterday: i64,
    pub avg_7d: f64,
    pub avg_14d: f64,
    pub avg_30d: f64,
    pub avg_time_today: f64,
    pub avg_time_yesterday: f64,
    pub avg_time_7d: f64,
    pub avg_time_14d: f64,
    pub avg_time_30d: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct AcquisitionRow {
    pub signups_today: i64,
    pub signups_yesterday: i64,
    pub signups_30d_avg: f64,
    pub logins_today: i64,
    pub logins_yesterday: i64,
    pub logins_30d_avg: f64,
    pub unique_today: i64,
    pub unique_yesterday: i64,
}

pub type OverviewRows = (SessionsRow, PageViewsRow, AcquisitionRow);

pub async fn fetch_overview_data(
    pool: &PgPool,
    today: chrono::NaiveDate,
    yesterday: chrono::NaiveDate,
) -> Result<OverviewRows, sqlx::Error> {
    let sessions_row = fetch_sessions(pool, today, yesterday).await?;
    let pv_row = fetch_page_views(pool, today, yesterday).await?;
    let acq_row = fetch_acquisition(pool, today, yesterday).await?;
    Ok((sessions_row, pv_row, acq_row))
}

async fn fetch_sessions(
    pool: &PgPool,
    today: chrono::NaiveDate,
    yesterday: chrono::NaiveDate,
) -> Result<SessionsRow, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE started_at::date = $1)::BIGINT as "today!",
            COUNT(*) FILTER (WHERE started_at::date = $2)::BIGINT as "yesterday!",
            COUNT(*) FILTER (WHERE started_at >= NOW() - INTERVAL '7 days')::FLOAT8 / 7.0 as "avg_7d!",
            COUNT(*) FILTER (WHERE started_at >= NOW() - INTERVAL '14 days')::FLOAT8 / 14.0 as "avg_14d!",
            COUNT(*) FILTER (WHERE started_at >= NOW() - INTERVAL '30 days')::FLOAT8 / 30.0 as "avg_30d!"
        FROM user_sessions
        WHERE started_at >= NOW() - INTERVAL '31 days'
          AND NOT is_bot AND NOT is_scanner
          AND NOT COALESCE(is_behavioral_bot, false)
          AND request_count > 0
        "#,
        today,
        yesterday,
    )
    .fetch_one(pool)
    .await?;
    Ok(SessionsRow {
        today: row.today,
        yesterday: row.yesterday,
        avg_7d: row.avg_7d,
        avg_14d: row.avg_14d,
        avg_30d: row.avg_30d,
    })
}

async fn fetch_page_views(
    pool: &PgPool,
    today: chrono::NaiveDate,
    yesterday: chrono::NaiveDate,
) -> Result<PageViewsRow, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE ee.created_at::date = $1)::BIGINT as "today!",
            COUNT(*) FILTER (WHERE ee.created_at::date = $2)::BIGINT as "yesterday!",
            COUNT(*) FILTER (WHERE ee.created_at >= NOW() - INTERVAL '7 days')::FLOAT8 / 7.0 as "avg_7d!",
            COUNT(*) FILTER (WHERE ee.created_at >= NOW() - INTERVAL '14 days')::FLOAT8 / 14.0 as "avg_14d!",
            COUNT(*) FILTER (WHERE ee.created_at >= NOW() - INTERVAL '30 days')::FLOAT8 / 30.0 as "avg_30d!",
            COALESCE(AVG(LEAST(NULLIF(ee.time_on_page_ms, 0), 600000)) FILTER (WHERE ee.created_at::date = $1), 0.0)::FLOAT8 as "avg_time_today!",
            COALESCE(AVG(LEAST(NULLIF(ee.time_on_page_ms, 0), 600000)) FILTER (WHERE ee.created_at::date = $2), 0.0)::FLOAT8 as "avg_time_yesterday!",
            COALESCE(AVG(LEAST(NULLIF(ee.time_on_page_ms, 0), 600000)) FILTER (WHERE ee.created_at >= NOW() - INTERVAL '7 days'), 0.0)::FLOAT8 as "avg_time_7d!",
            COALESCE(AVG(LEAST(NULLIF(ee.time_on_page_ms, 0), 600000)) FILTER (WHERE ee.created_at >= NOW() - INTERVAL '14 days'), 0.0)::FLOAT8 as "avg_time_14d!",
            COALESCE(AVG(LEAST(NULLIF(ee.time_on_page_ms, 0), 600000)) FILTER (WHERE ee.created_at >= NOW() - INTERVAL '30 days'), 0.0)::FLOAT8 as "avg_time_30d!"
        FROM engagement_events ee
        JOIN user_sessions us ON us.session_id = ee.session_id
        WHERE ee.created_at >= NOW() - INTERVAL '31 days'
          AND NOT us.is_bot AND NOT us.is_scanner
          AND NOT COALESCE(us.is_behavioral_bot, false)
          AND us.request_count > 0
        "#,
        today,
        yesterday,
    )
    .fetch_one(pool)
    .await?;
    Ok(PageViewsRow {
        today: row.today,
        yesterday: row.yesterday,
        avg_7d: row.avg_7d,
        avg_14d: row.avg_14d,
        avg_30d: row.avg_30d,
        avg_time_today: row.avg_time_today,
        avg_time_yesterday: row.avg_time_yesterday,
        avg_time_7d: row.avg_time_7d,
        avg_time_14d: row.avg_time_14d,
        avg_time_30d: row.avg_time_30d,
    })
}

async fn fetch_acquisition(
    pool: &PgPool,
    today: chrono::NaiveDate,
    yesterday: chrono::NaiveDate,
) -> Result<AcquisitionRow, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN event_type = 'user_created' AND remote_created_at::date = $1 THEN 1 ELSE 0 END), 0)::BIGINT as "signups_today!",
            COALESCE(SUM(CASE WHEN event_type = 'user_created' AND remote_created_at::date = $2 THEN 1 ELSE 0 END), 0)::BIGINT as "signups_yesterday!",
            COALESCE(SUM(CASE WHEN event_type = 'user_created' THEN 1 ELSE 0 END), 0)::FLOAT8 / 30.0 as "signups_30d_avg!",
            COALESCE(SUM(CASE WHEN event_type IN ('session_created', 'login') AND remote_created_at::date = $1 THEN 1 ELSE 0 END), 0)::BIGINT as "logins_today!",
            COALESCE(SUM(CASE WHEN event_type IN ('session_created', 'login') AND remote_created_at::date = $2 THEN 1 ELSE 0 END), 0)::BIGINT as "logins_yesterday!",
            COALESCE(SUM(CASE WHEN event_type IN ('session_created', 'login') THEN 1 ELSE 0 END), 0)::FLOAT8 / 30.0 as "logins_30d_avg!",
            COUNT(DISTINCT CASE WHEN remote_created_at::date = $1 THEN user_id END)::BIGINT as "unique_today!",
            COUNT(DISTINCT CASE WHEN remote_created_at::date = $2 THEN user_id END)::BIGINT as "unique_yesterday!"
        FROM tenant_activity
        WHERE remote_created_at >= NOW() - INTERVAL '31 days'
        "#,
        today,
        yesterday,
    )
    .fetch_one(pool)
    .await?;
    Ok(AcquisitionRow {
        signups_today: row.signups_today,
        signups_yesterday: row.signups_yesterday,
        signups_30d_avg: row.signups_30d_avg,
        logins_today: row.logins_today,
        logins_yesterday: row.logins_yesterday,
        logins_30d_avg: row.logins_30d_avg,
        unique_today: row.unique_today,
        unique_yesterday: row.unique_yesterday,
    })
}
