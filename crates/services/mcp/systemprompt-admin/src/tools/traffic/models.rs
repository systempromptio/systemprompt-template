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
pub struct DeviceBreakdownWithTrends {
    pub device_type: String,
    pub sessions: i32,
    pub percentage: f64,
    pub traffic_1d: i32,
    pub traffic_7d: i32,
    pub traffic_30d: i32,
}

#[derive(serde::Serialize)]
pub struct GeographicBreakdown {
    pub country: String,
    pub sessions: i32,
    pub percentage: f64,
    pub traffic_1d: i32,
    pub traffic_7d: i32,
    pub traffic_30d: i32,
}

#[derive(serde::Serialize)]
pub struct BrowserBreakdown {
    pub browser: String,
    pub sessions: i32,
    pub percentage: f64,
    pub traffic_1d: i32,
    pub traffic_7d: i32,
    pub traffic_30d: i32,
}

#[derive(serde::Serialize)]
pub struct OsBreakdown {
    pub os: String,
    pub sessions: i32,
    pub percentage: f64,
    pub traffic_1d: i32,
    pub traffic_7d: i32,
    pub traffic_30d: i32,
}

#[derive(serde::Serialize)]
pub struct Referrer {
    pub referrer_url: String,
    pub sessions: i32,
    pub unique_visitors: i32,
    pub avg_pages_per_session: f64,
    pub avg_duration_sec: f64,
}
