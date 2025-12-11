use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;

use super::models::{
    BrowserBreakdown, DeviceBreakdownWithTrends, GeographicBreakdown, OsBreakdown, Referrer,
    TrafficSummary,
};

pub struct TrafficRepository {
    pool: Arc<PgPool>,
}

impl TrafficRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }

    pub async fn get_traffic_summary(&self, days: i32) -> Result<TrafficSummary> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(DISTINCT session_id) as total_sessions,
                SUM(request_count)::bigint as total_requests,
                COUNT(DISTINCT user_id) as unique_users,
                AVG(EXTRACT(EPOCH FROM (last_activity_at - started_at)))::float8 as avg_session_duration_secs,
                AVG(request_count)::float8 as avg_requests_per_session,
                COALESCE(SUM(total_ai_cost_cents), 0)::bigint as total_cost_cents
            FROM user_sessions
            WHERE started_at >= NOW() - ($1 || ' days')::INTERVAL
            "#,
            days.to_string()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(TrafficSummary {
            total_sessions: row.total_sessions.unwrap_or(0) as i32,
            total_requests: row.total_requests.unwrap_or(0) as i32,
            unique_users: row.unique_users.unwrap_or(0) as i32,
            avg_session_duration_secs: row.avg_session_duration_secs.unwrap_or(0.0),
            avg_requests_per_session: row.avg_requests_per_session.unwrap_or(0.0),
            total_cost_cents: row.total_cost_cents.unwrap_or(0) as i32,
        })
    }

    pub async fn get_device_breakdown_with_trends(
        &self,
        days: i32,
    ) -> Result<Vec<DeviceBreakdownWithTrends>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                COALESCE(device_type, 'unknown') as device_type,
                COUNT(*) as sessions,
                (COUNT(*)::float / NULLIF(SUM(COUNT(*)) OVER(), 0) * 100)::float8 as percentage,
                COUNT(*) FILTER (WHERE started_at >= NOW() - INTERVAL '1 day') as traffic_1d,
                COUNT(*) FILTER (WHERE started_at >= NOW() - INTERVAL '7 days') as traffic_7d,
                COUNT(*) FILTER (WHERE started_at >= NOW() - INTERVAL '30 days') as traffic_30d
            FROM user_sessions
            WHERE started_at >= NOW() - ($1 || ' days')::INTERVAL
            GROUP BY device_type
            ORDER BY sessions DESC
            "#,
            days.to_string()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| DeviceBreakdownWithTrends {
                device_type: r.device_type.unwrap_or_else(|| "unknown".to_string()),
                sessions: r.sessions.unwrap_or(0) as i32,
                percentage: r.percentage.unwrap_or(0.0),
                traffic_1d: r.traffic_1d.unwrap_or(0) as i32,
                traffic_7d: r.traffic_7d.unwrap_or(0) as i32,
                traffic_30d: r.traffic_30d.unwrap_or(0) as i32,
            })
            .collect())
    }

    pub async fn get_geographic_breakdown(&self, days: i32) -> Result<Vec<GeographicBreakdown>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                COALESCE(country, 'Unknown') as country,
                COUNT(*) as sessions,
                (COUNT(*)::float / NULLIF(SUM(COUNT(*)) OVER(), 0) * 100)::float8 as percentage,
                COUNT(*) FILTER (WHERE started_at >= NOW() - INTERVAL '1 day') as traffic_1d,
                COUNT(*) FILTER (WHERE started_at >= NOW() - INTERVAL '7 days') as traffic_7d,
                COUNT(*) FILTER (WHERE started_at >= NOW() - INTERVAL '30 days') as traffic_30d
            FROM user_sessions
            WHERE started_at >= NOW() - ($1 || ' days')::INTERVAL
            GROUP BY country
            ORDER BY sessions DESC
            LIMIT 20
            "#,
            days.to_string()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| GeographicBreakdown {
                country: r.country.unwrap_or_else(|| "Unknown".to_string()),
                sessions: r.sessions.unwrap_or(0) as i32,
                percentage: r.percentage.unwrap_or(0.0),
                traffic_1d: r.traffic_1d.unwrap_or(0) as i32,
                traffic_7d: r.traffic_7d.unwrap_or(0) as i32,
                traffic_30d: r.traffic_30d.unwrap_or(0) as i32,
            })
            .collect())
    }

    pub async fn get_browser_breakdown(&self, days: i32) -> Result<Vec<BrowserBreakdown>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                COALESCE(browser, 'Unknown') as browser,
                COUNT(*) as sessions,
                (COUNT(*)::float / NULLIF(SUM(COUNT(*)) OVER(), 0) * 100)::float8 as percentage,
                COUNT(*) FILTER (WHERE started_at >= NOW() - INTERVAL '1 day') as traffic_1d,
                COUNT(*) FILTER (WHERE started_at >= NOW() - INTERVAL '7 days') as traffic_7d,
                COUNT(*) FILTER (WHERE started_at >= NOW() - INTERVAL '30 days') as traffic_30d
            FROM user_sessions
            WHERE started_at >= NOW() - ($1 || ' days')::INTERVAL
            GROUP BY browser
            ORDER BY sessions DESC
            LIMIT 10
            "#,
            days.to_string()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| BrowserBreakdown {
                browser: r.browser.unwrap_or_else(|| "Unknown".to_string()),
                sessions: r.sessions.unwrap_or(0) as i32,
                percentage: r.percentage.unwrap_or(0.0),
                traffic_1d: r.traffic_1d.unwrap_or(0) as i32,
                traffic_7d: r.traffic_7d.unwrap_or(0) as i32,
                traffic_30d: r.traffic_30d.unwrap_or(0) as i32,
            })
            .collect())
    }

    pub async fn get_os_breakdown(&self, days: i32) -> Result<Vec<OsBreakdown>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                COALESCE(os, 'Unknown') as os,
                COUNT(*) as sessions,
                (COUNT(*)::float / NULLIF(SUM(COUNT(*)) OVER(), 0) * 100)::float8 as percentage,
                COUNT(*) FILTER (WHERE started_at >= NOW() - INTERVAL '1 day') as traffic_1d,
                COUNT(*) FILTER (WHERE started_at >= NOW() - INTERVAL '7 days') as traffic_7d,
                COUNT(*) FILTER (WHERE started_at >= NOW() - INTERVAL '30 days') as traffic_30d
            FROM user_sessions
            WHERE started_at >= NOW() - ($1 || ' days')::INTERVAL
            GROUP BY os
            ORDER BY sessions DESC
            LIMIT 10
            "#,
            days.to_string()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| OsBreakdown {
                os: r.os.unwrap_or_else(|| "Unknown".to_string()),
                sessions: r.sessions.unwrap_or(0) as i32,
                percentage: r.percentage.unwrap_or(0.0),
                traffic_1d: r.traffic_1d.unwrap_or(0) as i32,
                traffic_7d: r.traffic_7d.unwrap_or(0) as i32,
                traffic_30d: r.traffic_30d.unwrap_or(0) as i32,
            })
            .collect())
    }

    pub async fn get_normalized_referrers(&self, days: i32) -> Result<Vec<Referrer>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                COALESCE(referrer_url, 'Direct') as referrer_url,
                COUNT(DISTINCT session_id) as sessions,
                COUNT(DISTINCT fingerprint_hash) as unique_visitors,
                AVG(request_count)::float8 as avg_pages_per_session,
                AVG(EXTRACT(EPOCH FROM (last_activity_at - started_at)))::float8 as avg_duration_sec
            FROM user_sessions
            WHERE started_at >= NOW() - ($1 || ' days')::INTERVAL
            GROUP BY referrer_url
            ORDER BY sessions DESC
            LIMIT 20
            "#,
            days.to_string()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Referrer {
                referrer_url: r.referrer_url.unwrap_or_else(|| "Direct".to_string()),
                sessions: r.sessions.unwrap_or(0) as i32,
                unique_visitors: r.unique_visitors.unwrap_or(0) as i32,
                avg_pages_per_session: r.avg_pages_per_session.unwrap_or(0.0),
                avg_duration_sec: r.avg_duration_sec.unwrap_or(0.0),
            })
            .collect())
    }
}
