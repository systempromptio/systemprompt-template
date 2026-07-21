//! Row shapes and queries backing each section of the analytics report.

#[derive(Debug, Clone)]
pub struct TopContentRow {
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

#[derive(Debug, Clone, Copy)]
pub struct SeoRow {
    pub total_impressions: i64,
    pub total_clicks: i64,
    pub total_indexed: i64,
    pub avg_position: f64,
}

#[derive(Debug, Clone)]
pub struct GeoRow {
    pub country: String,
    pub sessions: i64,
}

#[derive(Debug, Clone)]
pub struct DeviceSessionsRow {
    pub device: String,
    pub sessions: i64,
}

#[derive(Debug, Clone)]
pub struct SourceRow {
    pub source: String,
    pub sessions: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct FunnelRow {
    pub total_published: i64,
    pub avg_views: f64,
    pub total_shares: i64,
    pub total_comments: i64,
}

#[derive(Debug, Clone)]
pub struct LandingRow {
    pub page_url: String,
    pub sessions: i64,
    pub avg_time_seconds: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct SparkSessionRow {
    pub day: chrono::NaiveDate,
    pub sessions: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct SparkSignupRow {
    pub day: chrono::NaiveDate,
    pub signups: i64,
}

pub type ContentBreakdownResult = (
    Vec<TopContentRow>,
    SeoRow,
    Vec<GeoRow>,
    Vec<DeviceSessionsRow>,
    Vec<SourceRow>,
);
