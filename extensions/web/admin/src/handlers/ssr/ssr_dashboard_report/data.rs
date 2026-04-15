use sqlx::PgPool;

use super::error::DashboardResult;
use super::metrics::i64_to_f64;
use super::types::{SeoMetrics, TrafficOverviewData, UserAcquisitionData};

pub(super) struct SessionsRow {
    pub today: i64,
    pub yesterday: i64,
    pub avg_7d: f64,
    pub avg_14d: f64,
    pub avg_30d: f64,
}

pub(super) struct PageViewsRow {
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

pub(super) struct AcquisitionRow {
    pub signups_today: i64,
    pub signups_yesterday: i64,
    pub signups_30d_avg: f64,
    pub logins_today: i64,
    pub logins_yesterday: i64,
    pub logins_30d_avg: f64,
    pub unique_today: i64,
    pub unique_yesterday: i64,
}

pub(super) struct SeoRow {
    pub total_impressions: i64,
    pub total_clicks: i64,
    pub total_indexed: i64,
    pub avg_position: f64,
}

pub(super) struct TopContentRow {
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

pub(super) struct GeoRow {
    pub country: String,
    pub sessions: i64,
}

pub(super) struct DeviceRow {
    pub device: String,
    pub sessions: i64,
}

pub(super) struct SourceRow {
    pub source: String,
    pub sessions: i64,
}

pub(super) struct FunnelRow {
    pub total_published: i64,
    pub avg_views: f64,
    pub total_shares: i64,
    pub total_comments: i64,
}

pub(super) struct LandingRow {
    pub page_url: String,
    pub sessions: i64,
    pub avg_time_seconds: f64,
}

pub(super) struct SparkSessionRow {
    pub day: chrono::NaiveDate,
    pub sessions: i64,
}

pub(super) struct SparkSignupRow {
    pub day: chrono::NaiveDate,
    pub signups: i64,
}

pub(super) type ContentBreakdownResult = (
    Vec<TopContentRow>,
    SeoRow,
    Vec<GeoRow>,
    Vec<DeviceRow>,
    Vec<SourceRow>,
);

pub(super) fn build_traffic_overview(
    sessions_row: &SessionsRow,
    pv_row: &PageViewsRow,
) -> TrafficOverviewData {
    TrafficOverviewData {
        sessions_today: i64_to_f64(sessions_row.today),
        sessions_yesterday: i64_to_f64(sessions_row.yesterday),
        sessions_7d_avg: sessions_row.avg_7d,
        sessions_14d_avg: sessions_row.avg_14d,
        sessions_30d_avg: sessions_row.avg_30d,
        page_views_today: i64_to_f64(pv_row.today),
        page_views_yesterday: i64_to_f64(pv_row.yesterday),
        page_views_7d_avg: pv_row.avg_7d,
        page_views_14d_avg: pv_row.avg_14d,
        page_views_30d_avg: pv_row.avg_30d,
        unique_visitors_today: i64_to_f64(sessions_row.today),
        unique_visitors_yesterday: i64_to_f64(sessions_row.yesterday),
        unique_visitors_7d_avg: sessions_row.avg_7d,
        unique_visitors_14d_avg: sessions_row.avg_14d,
        unique_visitors_30d_avg: sessions_row.avg_30d,
        avg_time_ms_today: pv_row.avg_time_today,
        avg_time_ms_yesterday: pv_row.avg_time_yesterday,
        avg_time_ms_7d_avg: pv_row.avg_time_7d,
        avg_time_ms_14d_avg: pv_row.avg_time_14d,
        avg_time_ms_30d_avg: pv_row.avg_time_30d,
        ..TrafficOverviewData::default()
    }
}

pub(super) fn build_user_acquisition(acq_row: &AcquisitionRow) -> UserAcquisitionData {
    UserAcquisitionData {
        signups_today: i64_to_f64(acq_row.signups_today),
        signups_yesterday: i64_to_f64(acq_row.signups_yesterday),
        signups_30d_avg: acq_row.signups_30d_avg,
        logins_today: i64_to_f64(acq_row.logins_today),
        logins_yesterday: i64_to_f64(acq_row.logins_yesterday),
        logins_30d_avg: acq_row.logins_30d_avg,
        unique_users_today: i64_to_f64(acq_row.unique_today),
        unique_users_yesterday: i64_to_f64(acq_row.unique_yesterday),
        ..UserAcquisitionData::default()
    }
}

pub(super) fn build_seo_metrics(seo: &SeoRow) -> SeoMetrics {
    let seo_ctr = if seo.total_impressions > 0 {
        i64_to_f64(seo.total_clicks) / i64_to_f64(seo.total_impressions) * 100.0
    } else {
        0.0
    };
    SeoMetrics {
        total_impressions: seo.total_impressions,
        total_clicks: seo.total_clicks,
        avg_ctr: seo_ctr,
        total_indexed_pages: seo.total_indexed,
        avg_search_position: seo.avg_position,
    }
}

pub(super) async fn fetch_overview_data(
    pool: &PgPool,
    today: chrono::NaiveDate,
    yesterday: chrono::NaiveDate,
) -> DashboardResult<(SessionsRow, PageViewsRow, AcquisitionRow)> {
    let sessions_row = sqlx::query!(
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

    let pv_row = sqlx::query!(
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

    let acq_row = sqlx::query!(
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

    Ok((
        SessionsRow {
            today: sessions_row.today,
            yesterday: sessions_row.yesterday,
            avg_7d: sessions_row.avg_7d,
            avg_14d: sessions_row.avg_14d,
            avg_30d: sessions_row.avg_30d,
        },
        PageViewsRow {
            today: pv_row.today,
            yesterday: pv_row.yesterday,
            avg_7d: pv_row.avg_7d,
            avg_14d: pv_row.avg_14d,
            avg_30d: pv_row.avg_30d,
            avg_time_today: pv_row.avg_time_today,
            avg_time_yesterday: pv_row.avg_time_yesterday,
            avg_time_7d: pv_row.avg_time_7d,
            avg_time_14d: pv_row.avg_time_14d,
            avg_time_30d: pv_row.avg_time_30d,
        },
        AcquisitionRow {
            signups_today: acq_row.signups_today,
            signups_yesterday: acq_row.signups_yesterday,
            signups_30d_avg: acq_row.signups_30d_avg,
            logins_today: acq_row.logins_today,
            logins_yesterday: acq_row.logins_yesterday,
            logins_30d_avg: acq_row.logins_30d_avg,
            unique_today: acq_row.unique_today,
            unique_yesterday: acq_row.unique_yesterday,
        },
    ))
}
