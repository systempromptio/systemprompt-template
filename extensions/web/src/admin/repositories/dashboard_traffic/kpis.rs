use std::sync::Arc;

use sqlx::PgPool;

use crate::admin::types::TrafficKpis;

pub(super) async fn fetch_traffic_kpis(
    pool: &Arc<PgPool>,
    interval: &str,
    prev_interval: &str,
) -> Result<TrafficKpis, sqlx::Error> {
    let sessions = sqlx::query_as!(
        SessionKpis,
        r#"SELECT
            COUNT(*) FILTER (
                WHERE started_at >= NOW() - $1::text::interval
                AND NOT is_bot AND NOT is_scanner
                AND NOT COALESCE(is_behavioral_bot, false)
                AND request_count > 0
            )::BIGINT AS "current_sessions!",
            COUNT(*) FILTER (
                WHERE started_at >= NOW() - $2::text::interval
                AND started_at < NOW() - $1::text::interval
                AND NOT is_bot AND NOT is_scanner
                AND NOT COALESCE(is_behavioral_bot, false)
                AND request_count > 0
            )::BIGINT AS "previous_sessions!"
        FROM user_sessions
        WHERE started_at >= NOW() - $2::text::interval"#,
        interval,
        prev_interval,
    )
    .fetch_one(pool.as_ref())
    .await?;

    let engagement = sqlx::query_as!(
        EngagementKpis,
        r#"SELECT
            COUNT(*) FILTER (WHERE created_at >= NOW() - $1::text::interval AND time_on_page_ms > 0)::BIGINT AS "current_page_views!",
            COUNT(*) FILTER (WHERE created_at >= NOW() - $2::text::interval AND created_at < NOW() - $1::text::interval AND time_on_page_ms > 0)::BIGINT AS "previous_page_views!",
            COALESCE(AVG(LEAST(time_on_page_ms, 600000)) FILTER (WHERE created_at >= NOW() - $1::text::interval AND time_on_page_ms > 0), 0)::DOUBLE PRECISION AS "current_avg_time_ms!",
            COALESCE(AVG(LEAST(time_on_page_ms, 600000)) FILTER (WHERE created_at >= NOW() - $2::text::interval AND created_at < NOW() - $1::text::interval AND time_on_page_ms > 0), 0)::DOUBLE PRECISION AS "previous_avg_time_ms!",
            COALESCE(AVG(max_scroll_depth) FILTER (WHERE created_at >= NOW() - $1::text::interval AND time_on_page_ms > 0), 0)::DOUBLE PRECISION AS "current_avg_scroll!",
            COALESCE(AVG(max_scroll_depth) FILTER (WHERE created_at >= NOW() - $2::text::interval AND created_at < NOW() - $1::text::interval AND time_on_page_ms > 0), 0)::DOUBLE PRECISION AS "previous_avg_scroll!",
            COUNT(DISTINCT session_id) FILTER (WHERE created_at >= NOW() - $1::text::interval AND time_on_page_ms > 0)::BIGINT AS "current_unique_visitors!",
            COUNT(DISTINCT session_id) FILTER (WHERE created_at >= NOW() - $2::text::interval AND created_at < NOW() - $1::text::interval AND time_on_page_ms > 0)::BIGINT AS "previous_unique_visitors!"
        FROM engagement_events
        WHERE created_at >= NOW() - $2::text::interval
          AND time_on_page_ms > 0"#,
        interval,
        prev_interval,
    )
    .fetch_one(pool.as_ref())
    .await?;

    Ok(TrafficKpis {
        sessions_current: sessions.current_sessions,
        sessions_previous: sessions.previous_sessions,
        page_views_current: engagement.current_page_views,
        page_views_previous: engagement.previous_page_views,
        avg_time_ms_current: engagement.current_avg_time_ms,
        avg_time_ms_previous: engagement.previous_avg_time_ms,
        avg_scroll_current: engagement.current_avg_scroll,
        avg_scroll_previous: engagement.previous_avg_scroll,
        unique_visitors_current: engagement.current_unique_visitors,
        unique_visitors_previous: engagement.previous_unique_visitors,
    })
}

#[derive(Debug, sqlx::FromRow)]
struct SessionKpis {
    current_sessions: i64,
    previous_sessions: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct EngagementKpis {
    current_page_views: i64,
    previous_page_views: i64,
    current_avg_time_ms: f64,
    previous_avg_time_ms: f64,
    current_avg_scroll: f64,
    previous_avg_scroll: f64,
    current_unique_visitors: i64,
    previous_unique_visitors: i64,
}
