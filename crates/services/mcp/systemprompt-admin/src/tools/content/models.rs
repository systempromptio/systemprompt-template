use chrono::{DateTime, Utc};

#[derive(serde::Serialize, Clone)]
pub struct ContentPerformance {
    pub content_id: String,
    pub title: String,
    pub slug: String,
    pub source_id: String,
    pub published_at: Option<DateTime<Utc>>,
    pub days_old: i32,
    pub total_views: i32,
    pub unique_visitors: i32,
    pub preview_url: String,
    pub trackable_url: String,
}

#[derive(serde::Serialize)]
pub struct DailyViewData {
    pub content_id: String,
    pub title: String,
    pub view_date: String,
    pub daily_views: i32,
}

#[derive(serde::Serialize)]
pub struct Referrer {
    pub referrer_url: String,
    pub sessions: i32,
    pub unique_visitors: i32,
    pub avg_pages_per_session: f64,
    pub avg_duration_sec: f64,
}
