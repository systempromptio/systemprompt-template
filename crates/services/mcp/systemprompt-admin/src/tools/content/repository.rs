use anyhow::Result;
use systemprompt_core_database::{
    parse_database_datetime, DatabaseProvider, DatabaseQueryEnum, DbPool,
};

use super::models::{ContentPerformance, DailyViewData, Referrer};

const BASE_URL: &str = "https://tyingshoelaces.com";

pub struct ContentRepository {
    pool: DbPool,
}

impl ContentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn get_top_content(&self, days: i32) -> Result<Vec<ContentPerformance>> {
        let query = DatabaseQueryEnum::GetTopContent.get(self.pool.as_ref());
        let rows = self.pool.fetch_all(&query, &[&days]).await?;

        Ok(rows
            .iter()
            .map(|r| {
                let source_id = r
                    .get("source_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let slug = r
                    .get("slug")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let preview_url = build_preview_url(&source_id, &slug);
                ContentPerformance {
                    content_id: r
                        .get("content_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    title: r
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Untitled")
                        .to_string(),
                    slug: slug.clone(),
                    source_id: source_id.clone(),
                    published_at: r
                        .get("published_at")
                        .and_then(|v| parse_database_datetime(v)),
                    days_old: r.get("days_old").and_then(|v| v.as_f64()).unwrap_or(0.0) as i32,
                    total_views: r.get("total_views").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    unique_visitors: r
                        .get("unique_visitors")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32,
                    preview_url: preview_url.clone(),
                    trackable_url: preview_url,
                }
            })
            .collect())
    }

    pub async fn get_daily_views_per_content(&self, days: i32) -> Result<Vec<DailyViewData>> {
        let query = DatabaseQueryEnum::GetDailyViewsPerContent.get(self.pool.as_ref());
        let rows = self.pool.fetch_all(&query, &[&days]).await?;

        Ok(rows
            .iter()
            .map(|r| DailyViewData {
                content_id: r
                    .get("content_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                title: r
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                view_date: r
                    .get("view_date")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                daily_views: r.get("daily_views").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            })
            .collect())
    }

    pub async fn get_top_referrers(&self, days: i32) -> Result<Vec<Referrer>> {
        let query = DatabaseQueryEnum::GetTopReferrers.get(self.pool.as_ref());
        let rows = self.pool.fetch_all(&query, &[&days]).await?;

        Ok(rows
            .iter()
            .map(|r| Referrer {
                referrer_url: r
                    .get("referrer_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                sessions: r.get("sessions").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                unique_visitors: r
                    .get("unique_visitors")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                avg_pages_per_session: r
                    .get("avg_pages_per_session")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                avg_duration_sec: r
                    .get("avg_duration_sec")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            })
            .collect())
    }
}

fn build_preview_url(source_id: &str, slug: &str) -> String {
    match source_id {
        "blog" => format!("{}/blog/{}", BASE_URL, slug),
        "pages" => format!("{}/{}", BASE_URL, slug),
        _ => format!("{}/{}", BASE_URL, slug),
    }
}
