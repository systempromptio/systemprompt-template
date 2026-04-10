use std::fmt::Write;

use sqlx::PgPool;

use super::super::types::{
    ContentPerformanceRow, DashboardData, EventBreakdown, EventRow, EventsQuery, EventsResponse,
    RealtimePulse, RecentMcpError, TrafficData, TrafficTopPage,
};
use super::dashboard_aggregates::{
    fetch_active_users_24h, fetch_usage_timeseries, get_activity_stats,
};
use super::dashboard_queries::{
    fetch_hourly_activity, fetch_popular_skills, fetch_recent_mcp_errors, fetch_timeline,
    fetch_tool_success_rates, fetch_top_users,
};
use super::dashboard_traffic;

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
    let mut where_clause = String::from(
        " WHERE NOT ('anonymous' = ANY(u.roles)) AND u.email NOT LIKE '%@anonymous.local'",
    );
    let mut bind_idx = 0u32;
    let mut binds: Vec<String> = Vec::new();

    if let Some(ref search) = query.search {
        bind_idx += 1;
        let pattern = format!("%{search}%");
        let _ = write!(
            where_clause,
            " AND (p.id ILIKE ${bind_idx} OR u.display_name ILIKE ${bind_idx} OR u.email ILIKE ${bind_idx} OR p.tool_name ILIKE ${bind_idx} OR p.event_type ILIKE ${bind_idx} OR p.session_id ILIKE ${bind_idx})",
        );
        binds.push(pattern);
    }
    if let Some(ref et) = query.event_type {
        bind_idx += 1;
        let _ = write!(where_clause, " AND p.event_type = ${bind_idx}");
        binds.push(et.clone());
    }
    let count_sql = format!(
        "SELECT COALESCE(COUNT(*), 0)::BIGINT FROM plugin_usage_events p JOIN users u ON u.id = p.user_id{where_clause}"
    );
    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    for b in &binds {
        count_q = count_q.bind(b);
    }
    let total = count_q.fetch_one(pool).await?;

    let limit_idx = bind_idx + 1;
    let offset_idx = bind_idx + 2;

    let data_sql = format!(
        r"SELECT
            p.id, p.user_id,
            COALESCE(u.display_name, u.full_name, u.name, u.email, p.user_id) AS display_name,
            u.email, p.session_id, p.event_type, p.tool_name, p.plugin_id, p.metadata, p.created_at
        FROM plugin_usage_events p
        JOIN users u ON u.id = p.user_id
        {where_clause}
        ORDER BY p.created_at DESC
        LIMIT ${limit_idx} OFFSET ${offset_idx}"
    );

    let mut data_q = sqlx::query_as::<_, EventRow>(&data_sql);
    for b in &binds {
        data_q = data_q.bind(b);
    }
    data_q = data_q.bind(query.limit).bind(query.offset);
    let events = data_q.fetch_all(pool).await?;

    Ok(EventsResponse {
        events,
        total,
        limit: query.limit,
        offset: query.offset,
    })
}

pub async fn list_event_breakdown(pool: &PgPool) -> Result<Vec<EventBreakdown>, sqlx::Error> {
    let sql = r"SELECT p.event_type, COUNT(*)::BIGINT AS count
        FROM plugin_usage_events p
        JOIN users u ON u.id = p.user_id
        WHERE NOT ('anonymous' = ANY(u.roles)) AND u.email NOT LIKE '%@anonymous.local'
        GROUP BY p.event_type
        ORDER BY count DESC";
    sqlx::query_as::<_, EventBreakdown>(sql)
        .fetch_all(pool)
        .await
}
