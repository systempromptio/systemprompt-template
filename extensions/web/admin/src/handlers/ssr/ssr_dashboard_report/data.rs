pub(super) use crate::repositories::analytics_grp::dashboard_report::{
    fetch_overview_data, AcquisitionRow, DeviceRow, FunnelRow, GeoRow, LandingRow, PageViewsRow,
    SeoRow, SessionsRow, SourceRow, SparkSessionRow, SparkSignupRow, TopContentRow,
};

use super::metrics::i64_to_f64;
use super::types::{SeoMetrics, TrafficOverviewData, UserAcquisitionData};

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
