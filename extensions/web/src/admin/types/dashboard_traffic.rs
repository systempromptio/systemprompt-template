use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize)]
pub struct TrafficData {
    pub kpis: TrafficKpis,
    pub timeseries: Vec<TrafficTimeBucket>,
    pub sources: Vec<TrafficSource>,
    pub geo: Vec<TrafficGeo>,
    pub devices: Vec<TrafficDevice>,
    pub top_pages: Vec<TrafficTopPage>,
    pub country_timeseries: Vec<TrafficCountryBucket>,
    pub top_pages_daily: Vec<TopPageDailyBucket>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TopPageDailyBucket {
    pub page_url: String,
    pub day: NaiveDate,
    pub views: i64,
    pub sessions: i64,
    pub avg_time_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrafficKpis {
    pub sessions_current: i64,
    pub sessions_previous: i64,
    pub page_views_current: i64,
    pub page_views_previous: i64,
    pub avg_time_ms_current: f64,
    pub avg_time_ms_previous: f64,
    pub avg_scroll_current: f64,
    pub avg_scroll_previous: f64,
    pub unique_visitors_current: i64,
    pub unique_visitors_previous: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TrafficTimeBucket {
    pub bucket: DateTime<Utc>,
    pub sessions: i64,
    pub page_views: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TrafficSource {
    pub source: String,
    pub sessions: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TrafficGeo {
    pub country: String,
    pub sessions: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TrafficDevice {
    pub device: String,
    pub sessions: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TrafficTopPage {
    pub page_url: String,
    pub events: i64,
    pub sessions: i64,
    pub avg_time_ms: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TrafficReadingPattern {
    pub pattern: String,
    pub sessions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentMcpError {
    pub tool_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimePulse {
    pub sessions_this_hour: i64,
    pub page_views_this_hour: i64,
    pub unique_visitors_today: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPerformanceRow {
    pub title: String,
    pub views: i64,
    pub trend: Option<String>,
    pub avg_time_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TrafficCountryBucket {
    pub bucket: DateTime<Utc>,
    pub country: String,
    pub sessions: i64,
}
