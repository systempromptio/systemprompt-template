use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrafficOverviewData {
    pub sessions_today: f64,
    pub sessions_yesterday: f64,
    pub sessions_7d_avg: f64,
    pub sessions_14d_avg: f64,
    pub sessions_30d_avg: f64,
    pub page_views_today: f64,
    pub page_views_yesterday: f64,
    pub page_views_7d_avg: f64,
    pub page_views_14d_avg: f64,
    pub page_views_30d_avg: f64,
    pub unique_visitors_today: f64,
    pub unique_visitors_yesterday: f64,
    pub unique_visitors_7d_avg: f64,
    pub unique_visitors_14d_avg: f64,
    pub unique_visitors_30d_avg: f64,
    pub avg_time_ms_today: f64,
    pub avg_time_ms_yesterday: f64,
    pub avg_time_ms_7d_avg: f64,
    pub avg_time_ms_14d_avg: f64,
    pub avg_time_ms_30d_avg: f64,
    pub bounce_rate_today: f64,
    pub bounce_rate_yesterday: f64,
    pub bounce_rate_7d_avg: f64,
    pub bounce_rate_14d_avg: f64,
    pub bounce_rate_30d_avg: f64,
    pub pages_per_session_today: f64,
    pub pages_per_session_yesterday: f64,
    pub pages_per_session_7d_avg: f64,
    pub pages_per_session_14d_avg: f64,
    pub pages_per_session_30d_avg: f64,
}

pub type MetricTuple = (&'static str, f64, f64, f64, f64, f64, bool);

impl TrafficOverviewData {
    pub fn metric_values(&self) -> [MetricTuple; 6] {
        [
            (
                "Sessions",
                self.sessions_today,
                self.sessions_yesterday,
                self.sessions_7d_avg,
                self.sessions_14d_avg,
                self.sessions_30d_avg,
                true,
            ),
            (
                "Page Views",
                self.page_views_today,
                self.page_views_yesterday,
                self.page_views_7d_avg,
                self.page_views_14d_avg,
                self.page_views_30d_avg,
                true,
            ),
            (
                "Unique Visitors",
                self.unique_visitors_today,
                self.unique_visitors_yesterday,
                self.unique_visitors_7d_avg,
                self.unique_visitors_14d_avg,
                self.unique_visitors_30d_avg,
                true,
            ),
            (
                "Avg Time on Page",
                self.avg_time_ms_today,
                self.avg_time_ms_yesterday,
                self.avg_time_ms_7d_avg,
                self.avg_time_ms_14d_avg,
                self.avg_time_ms_30d_avg,
                true,
            ),
            (
                "Bounce Rate",
                self.bounce_rate_today,
                self.bounce_rate_yesterday,
                self.bounce_rate_7d_avg,
                self.bounce_rate_14d_avg,
                self.bounce_rate_30d_avg,
                false,
            ),
            (
                "Pages / Session",
                self.pages_per_session_today,
                self.pages_per_session_yesterday,
                self.pages_per_session_7d_avg,
                self.pages_per_session_14d_avg,
                self.pages_per_session_30d_avg,
                true,
            ),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserAcquisitionData {
    pub signups_today: f64,
    pub signups_yesterday: f64,
    pub signups_7d_avg: f64,
    pub signups_14d_avg: f64,
    pub signups_30d_avg: f64,
    pub logins_today: f64,
    pub logins_yesterday: f64,
    pub logins_7d_avg: f64,
    pub logins_14d_avg: f64,
    pub logins_30d_avg: f64,
    pub unique_users_today: f64,
    pub unique_users_yesterday: f64,
    pub unique_users_7d_avg: f64,
    pub unique_users_14d_avg: f64,
    pub unique_users_30d_avg: f64,
    pub conversion_rate_today: f64,
    pub conversion_rate_yesterday: f64,
    pub conversion_rate_7d_avg: f64,
    pub conversion_rate_14d_avg: f64,
    pub conversion_rate_30d_avg: f64,
}

impl UserAcquisitionData {
    pub fn metric_values(&self) -> [MetricTuple; 4] {
        [
            (
                "New Signups",
                self.signups_today,
                self.signups_yesterday,
                self.signups_7d_avg,
                self.signups_14d_avg,
                self.signups_30d_avg,
                true,
            ),
            (
                "Total Logins",
                self.logins_today,
                self.logins_yesterday,
                self.logins_7d_avg,
                self.logins_14d_avg,
                self.logins_30d_avg,
                true,
            ),
            (
                "Unique Active Users",
                self.unique_users_today,
                self.unique_users_yesterday,
                self.unique_users_7d_avg,
                self.unique_users_14d_avg,
                self.unique_users_30d_avg,
                true,
            ),
            (
                "Conversion Rate %",
                self.conversion_rate_today,
                self.conversion_rate_yesterday,
                self.conversion_rate_7d_avg,
                self.conversion_rate_14d_avg,
                self.conversion_rate_30d_avg,
                true,
            ),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopContentItem {
    pub title: String,
    #[serde(default)]
    pub slug: String,
    pub views_7d: i64,
    pub views_30d: i64,
    pub unique_visitors: i64,
    pub avg_time_seconds: f64,
    pub trend: String,
    pub search_impressions: i64,
    pub search_clicks: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SeoMetrics {
    pub total_impressions: i64,
    pub total_clicks: i64,
    pub avg_ctr: f64,
    pub total_indexed_pages: i64,
    pub avg_search_position: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoBreakdownItem {
    pub country: String,
    pub sessions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceBreakdownItem {
    pub device: String,
    pub sessions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceBreakdownItem {
    pub source: String,
    pub sessions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContentFunnel {
    pub total_published: i64,
    pub avg_views_per_piece: f64,
    pub total_shares: i64,
    pub total_comments: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SparklineData {
    pub sessions: Vec<i64>,
    pub page_views: Vec<i64>,
    pub signups: Vec<i64>,
    pub avg_time_ms: Vec<f64>,
    #[serde(default)]
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandingPageItem {
    pub page_url: String,
    pub sessions: i64,
    pub avg_time_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineReportData {
    pub traffic_overview: TrafficOverviewData,
    pub user_acquisition: UserAcquisitionData,
    pub top_content: Vec<TopContentItem>,
    pub seo_metrics: SeoMetrics,
    pub geo_breakdown: Vec<GeoBreakdownItem>,
    pub device_breakdown: Vec<DeviceBreakdownItem>,
    pub source_breakdown: Vec<SourceBreakdownItem>,
    pub content_funnel: ContentFunnel,
    pub sparklines: SparklineData,
    pub top_landing_pages: Vec<LandingPageItem>,
}
