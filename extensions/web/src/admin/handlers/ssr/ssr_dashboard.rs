use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{DashboardQuery, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use super::ssr_dashboard_types::{
    build_activity_data, build_mcp_health, ContentRangeFlags, DashboardTemplateData, McpErrorView,
    RangeFlags, TabFlags, TopPageView, TrafficRangeFlags,
};

struct DashboardCounts {
    total_users: usize,
    total_plugins: usize,
    total_skills: usize,
    agents_count: usize,
    mcp_count: usize,
}

struct ChartParams<'a> {
    interval: &'a str,
    bucket: &'a str,
    traffic_range: &'a str,
    content_range: &'a str,
}

async fn fetch_dashboard_data(
    pool: &Arc<PgPool>,
    services_path: Option<&std::path::PathBuf>,
    user_roles: &[String],
    chart: &ChartParams<'_>,
) -> (crate::admin::types::DashboardData, DashboardCounts) {
    let (dash_result, users_result, plugins_result) = tokio::join!(
        repositories::get_dashboard_data(
            pool,
            chart.interval,
            chart.bucket,
            chart.traffic_range,
            chart.content_range
        ),
        repositories::list_users(pool),
        async {
            match services_path.map(|p| repositories::list_plugins_for_roles(p, user_roles)) {
                Some(Ok(plugins)) => Ok(plugins),
                Some(Err(e)) => Err(e),
                None => Ok(vec![]),
            }
        },
    );

    let dash = dash_result.unwrap_or_else(|_| crate::admin::types::DashboardData {
        timeline: vec![],
        top_users: vec![],
        popular_skills: vec![],
        hourly_activity: vec![],
        stats: crate::admin::types::ActivityStats {
            events_today: 0,
            events_this_week: 0,
            total_edits: 0,
            mcp_tool_calls: 0,
            mcp_errors: 0,
            total_logins: 0,
        },
        usage_timeseries: vec![],
        active_users_24h: 0,
        tool_success_rates: vec![],
        traffic: None,
        recent_mcp_errors: vec![],
        top_pages_today: vec![],
        realtime_pulse: None,
        content_performance: vec![],
    });
    let users = users_result.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list users for dashboard");
        vec![]
    });
    let plugins = plugins_result.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list plugins for dashboard");
        vec![]
    });

    let counts = DashboardCounts {
        total_users: users.len(),
        total_plugins: plugins.len(),
        total_skills: plugins.iter().map(|p| p.skills.len()).sum(),
        agents_count: plugins.iter().map(|p| p.agents.len()).sum(),
        mcp_count: plugins.iter().map(|p| p.mcp_servers.len()).sum(),
    };

    (dash, counts)
}

pub(crate) async fn dashboard_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<DashboardQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return axum::response::Redirect::to("/admin/my/plugins").into_response();
    }

    let services_path = super::get_services_path()
        .map_err(|_| {
            tracing::warn!("Failed to get services path for dashboard");
        })
        .ok();
    let user_roles = user_ctx.roles.clone();

    let (interval_sql, bucket_sql, range_key) = match query.range.as_str() {
        "24h" => ("24 hours", "1 hour", "24h"),
        "14d" => ("14 days", "1 day", "14d"),
        _ => ("7 days", "4 hours", "7d"),
    };

    let traffic_range_key = match query.traffic_range.as_str() {
        "7d" => "7d",
        "30d" => "30d",
        _ => "today",
    };

    let content_range_key = match query.content_range.as_str() {
        "1h" => "1h",
        "24h" => "24h",
        "yesterday" => "yesterday",
        "30d" => "30d",
        _ => "7d",
    };

    let chart_params = ChartParams {
        interval: interval_sql,
        bucket: bucket_sql,
        traffic_range: traffic_range_key,
        content_range: content_range_key,
    };
    let (dash, counts) =
        fetch_dashboard_data(&pool, services_path.as_ref(), &user_roles, &chart_params).await;

    let tab = query.tab.as_str();
    let mut data = serde_json::to_value(build_dashboard_template(
        &dash,
        &counts,
        range_key,
        traffic_range_key,
        content_range_key,
        tab,
    ))
    .unwrap_or_default();

    if let Some(obj) = data.as_object_mut() {
        let users_val = obj.get("total_users").and_then(|v| v.as_u64()).unwrap_or(0);
        let active_val = obj
            .get("active_users_24h")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let events_val = obj
            .get("stats")
            .and_then(|s| s.get("events_today"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        obj.insert(
            "page_stats".to_string(),
            json!([
                {"value": users_val, "label": "Users", "key": "total_users"},
                {"value": active_val, "label": "Active Today", "key": "active_users_24h"},
                {"value": events_val, "label": "Events Today", "key": "events_today"},
            ]),
        );
    }

    if tab == "report" {
        if let Ok(Some(report_row)) =
            crate::admin::repositories::admin_traffic_reports::fetch_latest_report(&pool).await
        {
            let report = super::ssr_dashboard_report::build_dashboard_report(&report_row);
            if let Some(obj) = data.as_object_mut() {
                // JSON: protocol boundary
                obj.insert("has_dashboard_report".to_string(), json!(true));
                // JSON: protocol boundary
                obj.insert("dashboard_report".to_string(), report);
            }
        }
    }

    super::render_page(&engine, "dashboard", &data, &user_ctx, &mkt_ctx)
}

fn build_page_lists(
    dash: &crate::admin::types::DashboardData,
) -> (Vec<TopPageView>, Vec<McpErrorView>) {
    let top_pages_today = dash
        .top_pages_today
        .iter()
        .map(|p| TopPageView {
            page_url: p.page_url.clone(),
            events: p.events,
            sessions: p.sessions,
            avg_time: super::ssr_dashboard_traffic::format_time_ms(p.avg_time_ms),
        })
        .collect();
    let recent_mcp_errors = dash
        .recent_mcp_errors
        .iter()
        .map(|e| McpErrorView {
            tool_name: e.tool_name.clone(),
            created_at_display: e.created_at.format("%H:%M:%S").to_string(),
        })
        .collect();
    (top_pages_today, recent_mcp_errors)
}

fn build_dashboard_template(
    dash: &crate::admin::types::DashboardData,
    counts: &DashboardCounts,
    range_key: &str,
    traffic_range_key: &str,
    content_range_key: &str,
    tab: &str,
) -> DashboardTemplateData {
    let activity = build_activity_data(dash, range_key);
    let (mcp_error_rate_pct, mcp_health_status, mcp_health_label) = build_mcp_health(&dash.stats);
    let traffic_result = super::ssr_dashboard_traffic::build_traffic_data(
        dash.traffic.as_ref(),
        traffic_range_key,
        dash.realtime_pulse.as_ref(),
        &dash.content_performance,
    );
    let period_label = match traffic_range_key {
        "7d" => "vs previous 7 days",
        "30d" => "vs previous 30 days",
        _ => "vs yesterday",
    };
    let content_period_label = match content_range_key {
        "1h" => "1h",
        "24h" => "24h",
        "yesterday" => "Yesterday",
        "30d" => "30d",
        _ => "7d",
    };
    let active_tab = if tab.is_empty() { "traffic" } else { tab };
    let (top_pages_today, recent_mcp_errors) = build_page_lists(dash);

    DashboardTemplateData {
        page: "dashboard",
        title: "Dashboard",
        stats: serde_json::to_value(&dash.stats).unwrap_or_default(),
        timeline: serde_json::to_value(&dash.timeline).unwrap_or_default(),
        top_users: serde_json::to_value(&dash.top_users).unwrap_or_default(),
        popular_skills: activity.skills,
        hourly_activity: activity.hourly,
        total_users: counts.total_users,
        total_plugins: counts.total_plugins,
        total_skills: counts.total_skills,
        agents_count: counts.agents_count,
        mcp_count: counts.mcp_count,
        chart: activity.chart,
        range: range_key.to_string(),
        range_flags: RangeFlags {
            range_24h: range_key == "24h",
            range_7d: range_key == "7d",
            range_14d: range_key == "14d",
        },
        active_users_24h: dash.active_users_24h,
        error_rate_pct: mcp_error_rate_pct,
        tool_success_rates: activity.tools,
        traffic: traffic_result.has_traffic,
        traffic_range: traffic_range_key.to_string(),
        traffic_range_flags: TrafficRangeFlags {
            traffic_range_today: traffic_range_key == "today",
            traffic_range_7d: traffic_range_key == "7d",
            traffic_range_30d: traffic_range_key == "30d",
        },
        traffic_period_label: period_label,
        traffic_kpis: traffic_result.kpis,
        traffic_chart: traffic_result.chart,
        traffic_sources: traffic_result.sources,
        traffic_geo: traffic_result.geo,
        traffic_devices: traffic_result.devices,
        traffic_top_pages: traffic_result.top_pages,
        top_pages_enhanced: traffic_result.top_pages_enhanced,
        country_chart: traffic_result.country_chart,
        realtime_pulse: traffic_result.realtime_pulse,
        content_performance: traffic_result.content_performance,
        content_range: content_range_key.to_string(),
        content_range_flags: ContentRangeFlags {
            content_range_1h: content_range_key == "1h",
            content_range_24h: content_range_key == "24h",
            content_range_yesterday: content_range_key == "yesterday",
            content_range_7d: content_range_key == "7d",
            content_range_30d: content_range_key == "30d",
        },
        content_period_label,
        tab: active_tab.to_string(),
        tab_flags: TabFlags {
            tab_mcp: active_tab == "mcp",
            tab_traffic: active_tab == "traffic" || (active_tab != "mcp" && active_tab != "report"),
            tab_report: active_tab == "report",
        },
        active_tab: active_tab.to_string(),
        mcp_health_status,
        mcp_health_label,
        recent_mcp_errors,
        top_pages_today,
    }
}
