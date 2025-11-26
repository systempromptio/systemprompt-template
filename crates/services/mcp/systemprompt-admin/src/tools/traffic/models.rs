#[derive(serde::Serialize)]
pub struct TrafficSummary {
    pub total_sessions: i32,
    pub total_requests: i32,
    pub unique_users: i32,
    pub avg_session_duration_secs: f64,
    pub avg_requests_per_session: f64,
    pub total_cost_cents: i32,
}

#[derive(serde::Serialize)]
pub struct DeviceBreakdown {
    pub device_type: String,
    pub session_count: i32,
    pub request_count: i32,
    pub percentage: f64,
}

#[derive(serde::Serialize)]
pub struct TrafficSource {
    pub source_name: String,
    pub session_count: i32,
    pub unique_visitors: i32,
    pub avg_engagement_seconds: f64,
    pub bounce_rate: f64,
}

#[derive(serde::Serialize)]
pub struct LandingPage {
    pub landing_page: String,
    pub sessions: i32,
    pub unique_visitors: i32,
    pub bounce_rate: f64,
    pub avg_session_duration: f64,
}
