use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;

use super::models::{ContentPerformance, DailyViewData, Referrer, TrafficSummary};

const BASE_URL: &str = "https://tyingshoelaces.com";

pub struct ContentRepository {
    pool: Arc<PgPool>,
}

impl ContentRepository {
    pub fn new(db: DbPool) -> Result<Self> {
        let pool = db.pool_arc()?;
        Ok(Self { pool })
    }

    pub async fn get_daily_views_per_content(&self, days: i32) -> Result<Vec<DailyViewData>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                mc.id as content_id,
                mc.title,
                DATE(ae.timestamp)::text as view_date,
                COUNT(*) as daily_views
            FROM analytics_events ae
            JOIN markdown_content mc ON ae.endpoint = 'GET /blog/' || mc.slug
            JOIN user_sessions us ON ae.session_id = us.session_id
            WHERE ae.timestamp >= NOW() - ($1 || ' days')::INTERVAL
              AND ae.event_type = 'page_view'
              AND ae.endpoint LIKE 'GET /blog/%'
              AND us.is_bot = false
              AND us.is_scanner = false
            GROUP BY mc.id, mc.title, DATE(ae.timestamp)
            ORDER BY DATE(ae.timestamp) DESC, daily_views DESC
            "#,
            days.to_string()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| DailyViewData {
                content_id: r.content_id,
                title: r.title,
                view_date: r.view_date.unwrap_or_default(),
                daily_views: r.daily_views.unwrap_or(0) as i32,
            })
            .collect())
    }

    pub async fn get_traffic_summary(&self) -> Result<TrafficSummary> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(DISTINCT ae.session_id) FILTER (WHERE ae.timestamp >= NOW() - INTERVAL '1 day') as traffic_1d,
                COUNT(DISTINCT ae.session_id) FILTER (WHERE ae.timestamp >= NOW() - INTERVAL '7 days') as traffic_7d,
                COUNT(DISTINCT ae.session_id) FILTER (WHERE ae.timestamp >= NOW() - INTERVAL '30 days') as traffic_30d,
                COUNT(DISTINCT ae.session_id) FILTER (WHERE ae.timestamp >= NOW() - INTERVAL '2 days' AND ae.timestamp < NOW() - INTERVAL '1 day') as prev_traffic_1d,
                COUNT(DISTINCT ae.session_id) FILTER (WHERE ae.timestamp >= NOW() - INTERVAL '14 days' AND ae.timestamp < NOW() - INTERVAL '7 days') as prev_traffic_7d,
                COUNT(DISTINCT ae.session_id) FILTER (WHERE ae.timestamp >= NOW() - INTERVAL '60 days' AND ae.timestamp < NOW() - INTERVAL '30 days') as prev_traffic_30d
            FROM analytics_events ae
            JOIN user_sessions us ON ae.session_id = us.session_id
            WHERE ae.event_type = 'page_view'
              AND ae.endpoint LIKE 'GET /blog/%'
              AND us.is_bot = false
              AND us.is_scanner = false
            "#
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(TrafficSummary {
            traffic_1d: row.traffic_1d.unwrap_or(0) as i32,
            traffic_7d: row.traffic_7d.unwrap_or(0) as i32,
            traffic_30d: row.traffic_30d.unwrap_or(0) as i32,
            prev_traffic_1d: row.prev_traffic_1d.unwrap_or(0) as i32,
            prev_traffic_7d: row.prev_traffic_7d.unwrap_or(0) as i32,
            prev_traffic_30d: row.prev_traffic_30d.unwrap_or(0) as i32,
        })
    }

    pub async fn get_top_content_by_7d(&self, limit: i32) -> Result<Vec<ContentPerformance>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                mc.id as content_id,
                mc.title,
                mc.slug,
                mc.source_id,
                mc.published_at,
                EXTRACT(DAY FROM NOW() - mc.published_at)::integer as days_old,
                COUNT(ae.id) FILTER (WHERE us.session_id IS NOT NULL) as total_views,
                COUNT(DISTINCT ae.session_id) FILTER (WHERE us.session_id IS NOT NULL) as visitors_all_time,
                COUNT(DISTINCT ae.session_id) FILTER (WHERE ae.timestamp >= NOW() - INTERVAL '1 day' AND us.session_id IS NOT NULL) as visitors_1d,
                COUNT(DISTINCT ae.session_id) FILTER (WHERE ae.timestamp >= NOW() - INTERVAL '7 days' AND us.session_id IS NOT NULL) as visitors_7d,
                COUNT(DISTINCT ae.session_id) FILTER (WHERE ae.timestamp >= NOW() - INTERVAL '30 days' AND us.session_id IS NOT NULL) as visitors_30d
            FROM markdown_content mc
            LEFT JOIN analytics_events ae ON ae.endpoint = 'GET /blog/' || mc.slug
                AND ae.event_type = 'page_view'
            LEFT JOIN user_sessions us ON ae.session_id = us.session_id
                AND us.is_bot = false
                AND us.is_scanner = false
            GROUP BY mc.id, mc.title, mc.slug, mc.source_id, mc.published_at
            ORDER BY visitors_7d DESC NULLS LAST
            LIMIT $1
            "#,
            i64::from(limit)
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let source_id = r.source_id.clone();
                let slug = r.slug.clone();
                let preview_url = build_preview_url(&source_id, &slug);
                let published_at = Some(r.published_at);

                ContentPerformance {
                    content_id: r.content_id,
                    title: r.title,
                    slug,
                    source_id,
                    published_at,
                    days_old: r.days_old.unwrap_or(0),
                    total_views: r.total_views.unwrap_or(0) as i32,
                    visitors_all_time: r.visitors_all_time.unwrap_or(0) as i32,
                    visitors_1d: r.visitors_1d.unwrap_or(0) as i32,
                    visitors_7d: r.visitors_7d.unwrap_or(0) as i32,
                    visitors_30d: r.visitors_30d.unwrap_or(0) as i32,
                    preview_url: preview_url.clone(),
                    trackable_url: preview_url,
                }
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
              AND is_bot = false
              AND is_scanner = false
              AND request_count > 0
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

fn build_preview_url(source_id: &str, slug: &str) -> String {
    match source_id {
        "blog" => format!("{BASE_URL}/blog/{slug}"),
        "pages" => format!("{BASE_URL}/{slug}"),
        _ => format!("{BASE_URL}/{slug}"),
    }
}
