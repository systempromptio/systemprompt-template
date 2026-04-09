use super::ssr_dashboard_activity::{build_activity_data, build_mcp_health};
use super::ssr_dashboard_types::{
    DashboardTemplateData, McpErrorView, RangeFlags, TabFlags, TopPageView, TrafficRangeFlags,
};

pub(super) struct DashboardCounts {
    pub total_users: usize,
    pub total_plugins: usize,
    pub total_skills: usize,
    pub agents_count: usize,
    pub mcp_count: usize,
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

pub(super) fn build_dashboard_template(
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
    let active_tab = if tab.is_empty() { "governance" } else { tab };
    let (top_pages_today, recent_mcp_errors) = build_page_lists(dash);

    DashboardTemplateData {
        page: "dashboard",
        title: "Dashboard",
        stats: serde_json::to_value(&dash.stats).unwrap_or_else(|_| serde_json::Value::Null),
        timeline: serde_json::to_value(&dash.timeline).unwrap_or_else(|_| serde_json::Value::Null),
        top_users: serde_json::to_value(&dash.top_users)
            .unwrap_or_else(|_| serde_json::Value::Null),
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
        content_period_label,
        tab: active_tab.to_string(),
        tab_flags: TabFlags {
            tab_mcp: active_tab == "mcp",
            tab_traffic: active_tab == "governance"
                || (active_tab != "mcp" && active_tab != "report"),
            tab_report: active_tab == "report",
        },
        active_tab: active_tab.to_string(),
        mcp_health_status,
        mcp_health_label,
        recent_mcp_errors,
        top_pages_today,
    }
}
