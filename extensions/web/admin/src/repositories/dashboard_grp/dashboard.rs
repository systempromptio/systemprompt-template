use sqlx::PgPool;
use systemprompt::identifiers::{Email, PluginId, SessionId, UserId};

use crate::repositories::dashboard_aggregates::{
    fetch_active_users_24h, fetch_usage_timeseries, get_activity_stats,
};
use crate::repositories::dashboard_queries::{
    fetch_hourly_activity, fetch_popular_skills, fetch_recent_mcp_errors, fetch_timeline,
    fetch_tool_success_rates, fetch_top_users,
};
use crate::repositories::dashboard_traffic;
use crate::types::{
    ContentPerformanceRow, DashboardData, EventBreakdown, EventRow, EventsQuery, EventsResponse,
    RealtimePulse, RecentMcpError, TrafficData, TrafficTopPage,
};

pub async fn get_dashboard_data(
    pool: &PgPool,
    chart_interval: &str,
    chart_bucket: &str,
    traffic_range: &str,
    content_range: &str,
) -> Result<DashboardData, sqlx::Error> {
    let (
        timeline,
        top_users,
        popular_skills,
        hourly_activity,
        stats,
        usage_timeseries,
        active_users_24h,
        tool_success_rates,
    ) = tokio::try_join!(
        fetch_timeline(pool),
        fetch_top_users(pool),
        fetch_popular_skills(pool),
        fetch_hourly_activity(pool),
        get_activity_stats(pool),
        fetch_usage_timeseries(pool, chart_interval, chart_bucket),
        fetch_active_users_24h(pool),
        fetch_tool_success_rates(pool),
    )?;

    let (traffic, recent_mcp_errors, top_pages_today, realtime_pulse, content_performance) =
        fetch_traffic_section(pool, traffic_range, content_range).await;

    Ok(DashboardData {
        timeline,
        top_users,
        popular_skills,
        hourly_activity,
        stats,
        usage_timeseries,
        active_users_24h,
        tool_success_rates,
        traffic,
        recent_mcp_errors,
        top_pages_today,
        realtime_pulse,
        content_performance,
    })
}

type TrafficSectionResult = (
    Option<TrafficData>,
    Vec<RecentMcpError>,
    Vec<TrafficTopPage>,
    Option<RealtimePulse>,
    Vec<ContentPerformanceRow>,
);

async fn fetch_traffic_section(
    pool: &PgPool,
    traffic_range: &str,
    content_range: &str,
) -> TrafficSectionResult {
    tokio::join!(
        async {
            dashboard_traffic::fetch_traffic_data(pool, traffic_range)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "Failed to fetch traffic data for dashboard");
                    e
                })
                .ok()
        },
        async {
            fetch_recent_mcp_errors(pool).await.unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to fetch recent MCP errors");
                vec![]
            })
        },
        async {
            dashboard_traffic::fetch_top_pages_today(pool)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!(error = %e, "Failed to fetch top pages today");
                    vec![]
                })
        },
        async {
            dashboard_traffic::fetch_realtime_pulse(pool)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "Failed to fetch realtime pulse");
                    e
                })
                .ok()
        },
        async {
            dashboard_traffic::fetch_content_performance(pool, content_range)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!(error = %e, "Failed to fetch content performance");
                    vec![]
                })
        },
    )
}

pub async fn list_events(
    pool: &PgPool,
    query: &EventsQuery,
) -> Result<EventsResponse, sqlx::Error> {
    let search_pattern = query
        .search
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));
    let event_type = query.event_type.as_deref();

    let total = sqlx::query_scalar!(
        r#"SELECT COALESCE(COUNT(*), 0)::BIGINT AS "count!"
           FROM plugin_usage_events p
           JOIN users u ON u.id = p.user_id
           WHERE NOT ('anonymous' = ANY(u.roles)) AND u.email NOT LIKE '%@anonymous.local'
             AND ($1::text IS NULL
                  OR p.id ILIKE $1
                  OR u.display_name ILIKE $1
                  OR u.email ILIKE $1
                  OR p.tool_name ILIKE $1
                  OR p.event_type ILIKE $1
                  OR p.session_id ILIKE $1)
             AND ($2::text IS NULL OR p.event_type = $2)"#,
        search_pattern.as_deref(),
        event_type,
    )
    .fetch_one(pool)
    .await?;

    let events = sqlx::query_as!(
        EventRow,
        r#"SELECT
            p.id AS "id!",
            p.user_id AS "user_id!: UserId",
            COALESCE(u.display_name, u.full_name, u.name, u.email, p.user_id) AS "display_name!",
            u.email AS "email?: Email",
            p.session_id AS "session_id!: SessionId",
            p.event_type AS "event_type!",
            p.tool_name,
            p.plugin_id AS "plugin_id?: PluginId",
            p.metadata AS "metadata!",
            p.created_at AS "created_at!"
        FROM plugin_usage_events p
        JOIN users u ON u.id = p.user_id
        WHERE NOT ('anonymous' = ANY(u.roles)) AND u.email NOT LIKE '%@anonymous.local'
          AND ($1::text IS NULL
               OR p.id ILIKE $1
               OR u.display_name ILIKE $1
               OR u.email ILIKE $1
               OR p.tool_name ILIKE $1
               OR p.event_type ILIKE $1
               OR p.session_id ILIKE $1)
          AND ($2::text IS NULL OR p.event_type = $2)
        ORDER BY p.created_at DESC
        LIMIT $3 OFFSET $4"#,
        search_pattern.as_deref(),
        event_type,
        query.limit,
        query.offset,
    )
    .fetch_all(pool)
    .await?;

    Ok(EventsResponse {
        events,
        total,
        limit: query.limit,
        offset: query.offset,
    })
}

pub async fn list_event_breakdown(pool: &PgPool) -> Result<Vec<EventBreakdown>, sqlx::Error> {
    sqlx::query_as!(
        EventBreakdown,
        r#"SELECT p.event_type, COUNT(*)::BIGINT AS "count!"
           FROM plugin_usage_events p
           JOIN users u ON u.id = p.user_id
           WHERE NOT ('anonymous' = ANY(u.roles)) AND u.email NOT LIKE '%@anonymous.local'
           GROUP BY p.event_type
           ORDER BY COUNT(*) DESC"#,
    )
    .fetch_all(pool)
    .await
}
