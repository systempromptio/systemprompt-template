use std::fmt::Write;
use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::{DashboardData, EventRow, EventsQuery, EventsResponse};
use super::dashboard_aggregates::{
    fetch_active_users_24h, fetch_avg_session_duration, fetch_event_breakdown,
    fetch_usage_timeseries, get_activity_stats,
};
use super::dashboard_queries::{
    fetch_department_activity, fetch_hourly_activity, fetch_model_usage, fetch_popular_skills,
    fetch_project_activity, fetch_timeline, fetch_tool_success_rates, fetch_top_users,
};
use super::dashboard_queries_extra::{fetch_mcp_access_events, fetch_mcp_access_stats};

pub async fn get_dashboard_data(
    pool: &Arc<PgPool>,
    department: Option<&str>,
    chart_interval: &str,
) -> Result<DashboardData, sqlx::Error> {
    let (
        timeline,
        top_users,
        popular_skills,
        hourly_activity,
        stats,
        department_activity,
        model_usage,
        event_breakdown,
        usage_timeseries,
        active_users_24h,
        avg_session_duration_secs,
        project_activity,
        tool_success_rates,
        mcp_access_events,
        mcp_access_stats,
    ) = tokio::try_join!(
        fetch_timeline(pool, department),
        fetch_top_users(pool, department),
        fetch_popular_skills(pool, department),
        fetch_hourly_activity(pool, department),
        get_activity_stats(pool, department),
        fetch_department_activity(pool, department),
        fetch_model_usage(pool, department),
        fetch_event_breakdown(pool, department),
        fetch_usage_timeseries(pool, department, chart_interval),
        fetch_active_users_24h(pool, department),
        fetch_avg_session_duration(pool, department),
        fetch_project_activity(pool, department),
        fetch_tool_success_rates(pool, department),
        fetch_mcp_access_events(pool),
        fetch_mcp_access_stats(pool),
    )?;

    Ok(DashboardData {
        timeline,
        top_users,
        popular_skills,
        hourly_activity,
        department_activity,
        stats,
        model_usage,
        event_breakdown,
        usage_timeseries,
        active_users_24h,
        avg_session_duration_secs,
        project_activity,
        tool_success_rates,
        mcp_access_events,
        mcp_access_stats,
    })
}

pub async fn list_events(
    pool: &Arc<PgPool>,
    query: &EventsQuery,
    department: Option<&str>,
) -> Result<EventsResponse, sqlx::Error> {
    let mut where_clause = String::from(
        " WHERE NOT ('anonymous' = ANY(u.roles)) AND u.email NOT LIKE '%@anonymous.local'",
    );
    let mut bind_idx = 0u32;
    let mut binds: Vec<String> = Vec::new();

    if let Some(dept) = department {
        bind_idx += 1;
        write!(where_clause, " AND u.department = ${bind_idx}")
            .expect("write to String cannot fail");
        binds.push(dept.to_string());
    }

    if let Some(ref search) = query.search {
        bind_idx += 1;
        let pattern = format!("%{search}%");
        write!(
            where_clause,
            " AND (p.id ILIKE ${bind_idx} OR u.display_name ILIKE ${bind_idx} OR u.email ILIKE ${bind_idx} OR p.tool_name ILIKE ${bind_idx} OR p.event_type ILIKE ${bind_idx} OR p.session_id ILIKE ${bind_idx})",
        )
        .expect("write to String cannot fail");
        binds.push(pattern);
    }
    if let Some(ref et) = query.event_type {
        bind_idx += 1;
        write!(where_clause, " AND p.event_type = ${bind_idx}")
            .expect("write to String cannot fail");
        binds.push(et.clone());
    }
    if let Some(ref pid) = query.plugin_id {
        bind_idx += 1;
        write!(where_clause, " AND p.plugin_id = ${bind_idx}")
            .expect("write to String cannot fail");
        binds.push(pid.clone());
    }

    let count_sql = format!(
        "SELECT COALESCE(COUNT(*), 0)::BIGINT FROM plugin_usage_events p JOIN users u ON u.id = p.user_id{where_clause}"
    );
    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    for b in &binds {
        count_q = count_q.bind(b);
    }
    let total = count_q.fetch_one(pool.as_ref()).await?;

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
    let events = data_q.fetch_all(pool.as_ref()).await?;

    Ok(EventsResponse {
        events,
        total,
        limit: query.limit,
        offset: query.offset,
    })
}
