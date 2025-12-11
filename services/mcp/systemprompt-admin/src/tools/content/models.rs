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
    pub visitors_all_time: i32,
    pub visitors_1d: i32,
    pub visitors_7d: i32,
    pub visitors_30d: i32,
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

#[derive(serde::Serialize)]
pub struct TrafficSummary {
    pub traffic_1d: i32,
    pub traffic_7d: i32,
    pub traffic_30d: i32,
    pub prev_traffic_1d: i32,
    pub prev_traffic_7d: i32,
    pub prev_traffic_30d: i32,
}

impl TrafficSummary {
    pub fn diff_1d(&self) -> i32 {
        self.traffic_1d - self.prev_traffic_1d
    }

    pub fn diff_7d(&self) -> i32 {
        self.traffic_7d - self.prev_traffic_7d
    }

    pub fn diff_30d(&self) -> i32 {
        self.traffic_30d - self.prev_traffic_30d
    }

    pub fn percent_change_1d(&self) -> f64 {
        if self.prev_traffic_1d == 0 {
            0.0
        } else {
            ((self.traffic_1d - self.prev_traffic_1d) as f64 / self.prev_traffic_1d as f64) * 100.0
        }
    }

    pub fn percent_change_7d(&self) -> f64 {
        if self.prev_traffic_7d == 0 {
            0.0
        } else {
            ((self.traffic_7d - self.prev_traffic_7d) as f64 / self.prev_traffic_7d as f64) * 100.0
        }
    }

    pub fn percent_change_30d(&self) -> f64 {
        if self.prev_traffic_30d == 0 {
            0.0
        } else {
            ((self.traffic_30d - self.prev_traffic_30d) as f64 / self.prev_traffic_30d as f64)
                * 100.0
        }
    }
}
