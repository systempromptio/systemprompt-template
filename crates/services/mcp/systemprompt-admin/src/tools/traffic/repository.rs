use anyhow::Result;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbPool};

use super::models::{DeviceBreakdown, LandingPage, TrafficSource, TrafficSummary};

pub struct TrafficRepository {
    pool: DbPool,
}

impl TrafficRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn get_traffic_summary(&self, days: i32) -> Result<TrafficSummary> {
        let query = DatabaseQueryEnum::GetTrafficSummary.get(self.pool.as_ref());
        let row = self.pool.fetch_one(&query, &[&days]).await?;

        Ok(TrafficSummary {
            total_sessions: row
                .get("total_sessions")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            total_requests: row
                .get("total_requests")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            unique_users: row
                .get("unique_users")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            avg_session_duration_secs: row
                .get("avg_session_duration_secs")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            avg_requests_per_session: row
                .get("avg_requests_per_session")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            total_cost_cents: row
                .get("total_cost_cents")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
        })
    }

    pub async fn get_device_breakdown(&self, days: i32) -> Result<Vec<DeviceBreakdown>> {
        let query = DatabaseQueryEnum::GetDeviceBreakdown.get(self.pool.as_ref());
        let rows = self.pool.fetch_all(&query, &[&days]).await?;

        let total_sessions: i32 = rows
            .iter()
            .map(|r| r.get("session_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32)
            .sum();

        Ok(rows
            .iter()
            .map(|r| {
                let session_count =
                    r.get("session_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                DeviceBreakdown {
                    device_type: r
                        .get("device_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    session_count,
                    request_count: r.get("request_count").and_then(|v| v.as_i64()).unwrap_or(0)
                        as i32,
                    percentage: if total_sessions > 0 {
                        (session_count as f64 / total_sessions as f64) * 100.0
                    } else {
                        0.0
                    },
                }
            })
            .collect())
    }

    pub async fn get_traffic_sources(&self, days: i32) -> Result<Vec<TrafficSource>> {
        let query = DatabaseQueryEnum::GetTrafficSources.get(self.pool.as_ref());
        let rows = self.pool.fetch_all(&query, &[&days]).await?;

        Ok(rows
            .iter()
            .map(|r| TrafficSource {
                source_name: r
                    .get("source_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                session_count: r.get("session_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                unique_visitors: r
                    .get("unique_visitors")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                avg_engagement_seconds: r
                    .get("avg_engagement_seconds")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                bounce_rate: r.get("bounce_rate").and_then(|v| v.as_f64()).unwrap_or(0.0),
            })
            .collect())
    }

    pub async fn get_landing_pages(&self, days: i32) -> Result<Vec<LandingPage>> {
        let query = DatabaseQueryEnum::GetLandingPages.get(self.pool.as_ref());
        let rows = self.pool.fetch_all(&query, &[&days]).await?;

        Ok(rows
            .iter()
            .map(|r| LandingPage {
                landing_page: r
                    .get("landing_page")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(not set)")
                    .to_string(),
                sessions: r.get("sessions").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                unique_visitors: r
                    .get("unique_visitors")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                bounce_rate: r.get("bounce_rate").and_then(|v| v.as_f64()).unwrap_or(0.0),
                avg_session_duration: r
                    .get("avg_session_duration")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            })
            .collect())
    }
}
