use serde::Serialize;

use super::types::ContentFunnel;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MetricRowView {
    pub label: String,
    pub value: String,
    pub yesterday_delta: String,
    pub yesterday_arrow: String,
    pub yesterday_sentiment: String,
    pub week_delta: String,
    pub week_arrow: String,
    pub week_sentiment: String,
    pub fortnight_delta: String,
    pub fortnight_arrow: String,
    pub fortnight_sentiment: String,
    pub global_delta: String,
    pub global_arrow: String,
    pub global_sentiment: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TopContentView {
    pub title: String,
    pub views_7d: i64,
    pub views_30d: i64,
    pub unique_visitors: i64,
    pub avg_time: String,
    pub trend_icon: &'static str,
    pub trend_class: &'static str,
    pub search_impressions: i64,
    pub search_clicks: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BarDataView {
    pub label: String,
    pub sessions: i64,
    pub pct: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LandingPageView {
    pub page_url: String,
    pub sessions: i64,
    pub avg_time: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ContentVisibility {
    pub has_top_content: bool,
    pub has_landing_pages: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BreakdownVisibility {
    pub has_geo: bool,
    pub has_devices: bool,
    pub has_sources: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DashboardReportView {
    pub report_date: String,
    pub generated_at: String,
    pub traffic_overview: Vec<MetricRowView>,
    pub user_acquisition: Vec<MetricRowView>,
    pub top_content: Vec<TopContentView>,
    pub seo_impressions: i64,
    pub seo_clicks: i64,
    pub seo_ctr: String,
    pub seo_indexed: i64,
    pub seo_avg_position: String,
    pub geo_breakdown: Vec<BarDataView>,
    pub device_breakdown: Vec<BarDataView>,
    pub source_breakdown: Vec<BarDataView>,
    pub content_funnel: ContentFunnel,
    pub landing_pages: Vec<LandingPageView>,
    pub sparkline_sessions: String,
    pub sparkline_page_views: String,
    pub sparkline_signups: String,
    pub sparkline_avg_time: String,
    #[serde(flatten)]
    pub content_visibility: ContentVisibility,
    #[serde(flatten)]
    pub breakdown_visibility: BreakdownVisibility,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApiErrorResponse {
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApiStatusResponse {
    pub status: &'static str,
}

pub(crate) struct SparklineStrings {
    pub sessions: String,
    pub page_views: String,
    pub signups: String,
    pub avg_time: String,
}
