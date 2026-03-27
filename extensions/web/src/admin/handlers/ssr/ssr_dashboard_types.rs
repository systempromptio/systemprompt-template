use crate::admin::numeric;
use crate::admin::types;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(super) struct TrafficKpisView {
    pub sessions: i64,
    pub sessions_change: String,
    pub sessions_dir: String,
    pub sessions_class: String,
    pub page_views: i64,
    pub pv_change: String,
    pub pv_dir: String,
    pub pv_class: String,
    pub avg_time: String,
    pub time_change: String,
    pub time_dir: String,
    pub time_class: String,
    pub unique_visitors: i64,
    pub uv_change: String,
    pub uv_dir: String,
    pub uv_class: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct RealtimePulseView {
    pub sessions_this_hour: i64,
    pub page_views_this_hour: i64,
    pub unique_visitors_today: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ContentPerformanceView {
    pub title: String,
    pub views: i64,
    pub trend: Option<String>,
    pub avg_time: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct SourceBar {
    pub source: String,
    pub sessions: i64,
    pub pct: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct GeoBar {
    pub country: String,
    pub sessions: i64,
    pub pct: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct DeviceBar {
    pub device: String,
    pub sessions: i64,
    pub pct: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct TopPageView {
    pub page_url: String,
    pub events: i64,
    pub sessions: i64,
    pub avg_time: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct TopPageHorizon {
    pub views: i64,
    pub sessions: i64,
    pub avg_time: String,
    pub views_sparkline: String,
    pub views_trend: String,
    pub views_change: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct TopPageEnhancedView {
    pub page_url: String,
    pub page_label: String,
    pub horizon_1d: TopPageHorizon,
    pub horizon_yesterday: TopPageHorizon,
    pub horizon_7d: TopPageHorizon,
    pub horizon_31d: TopPageHorizon,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct TrafficResult {
    pub has_traffic: bool,
    pub kpis: Option<TrafficKpisView>,
    // JSON: required by trait contract
    pub chart: serde_json::Value,
    pub sources: Vec<SourceBar>,
    pub geo: Vec<GeoBar>,
    pub devices: Vec<DeviceBar>,
    pub top_pages: Vec<TopPageView>,
    pub top_pages_enhanced: Vec<TopPageEnhancedView>,
    // JSON: required by trait contract
    pub country_chart: serde_json::Value,
    pub realtime_pulse: Option<RealtimePulseView>,
    pub content_performance: Vec<ContentPerformanceView>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct SkillBar {
    pub tool_name: String,
    pub count: i64,
    pub pct: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ToolSuccessBar {
    pub tool_name: String,
    pub total: i64,
    pub successes: i64,
    pub failures: i64,
    pub success_pct: String,
    pub pct: f64,
    pub color_class: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct HourlyBar {
    pub hour: usize,
    pub count: i64,
    pub pct: i64,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct McpErrorView {
    pub tool_name: String,
    pub created_at_display: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct RangeFlags {
    pub range_24h: bool,
    pub range_7d: bool,
    pub range_14d: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct TrafficRangeFlags {
    pub traffic_range_today: bool,
    pub traffic_range_7d: bool,
    pub traffic_range_30d: bool,
}

#[derive(Debug, Clone, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub(super) struct ContentRangeFlags {
    pub content_range_1h: bool,
    pub content_range_24h: bool,
    pub content_range_yesterday: bool,
    pub content_range_7d: bool,
    pub content_range_30d: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct TabFlags {
    pub tab_mcp: bool,
    pub tab_traffic: bool,
    pub tab_report: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct DashboardTemplateData {
    pub page: &'static str,
    pub title: &'static str,
    // JSON: required by trait contract
    pub stats: serde_json::Value,
    // JSON: required by trait contract
    pub timeline: serde_json::Value,
    // JSON: required by trait contract
    pub top_users: serde_json::Value,
    pub popular_skills: Vec<SkillBar>,
    pub hourly_activity: Vec<HourlyBar>,
    pub total_users: usize,
    pub total_plugins: usize,
    pub total_skills: usize,
    pub agents_count: usize,
    pub mcp_count: usize,
    // JSON: required by trait contract
    pub chart: serde_json::Value,
    pub range: String,
    #[serde(flatten)]
    pub range_flags: RangeFlags,
    pub active_users_24h: i64,
    pub error_rate_pct: usize,
    pub tool_success_rates: Vec<ToolSuccessBar>,
    pub traffic: bool,
    pub traffic_range: String,
    #[serde(flatten)]
    pub traffic_range_flags: TrafficRangeFlags,
    pub traffic_period_label: &'static str,
    pub traffic_kpis: Option<TrafficKpisView>,
    // JSON: required by trait contract
    pub traffic_chart: serde_json::Value,
    pub traffic_sources: Vec<SourceBar>,
    pub traffic_geo: Vec<GeoBar>,
    pub traffic_devices: Vec<DeviceBar>,
    pub traffic_top_pages: Vec<TopPageView>,
    pub top_pages_enhanced: Vec<TopPageEnhancedView>,
    // JSON: required by trait contract
    pub country_chart: serde_json::Value,
    pub realtime_pulse: Option<RealtimePulseView>,
    pub content_performance: Vec<ContentPerformanceView>,
    pub content_range: String,
    #[serde(flatten)]
    pub content_range_flags: ContentRangeFlags,
    pub content_period_label: &'static str,
    pub tab: String,
    #[serde(flatten)]
    pub tab_flags: TabFlags,
    pub active_tab: String,
    pub mcp_health_status: &'static str,
    pub mcp_health_label: &'static str,
    pub recent_mcp_errors: Vec<McpErrorView>,
    pub top_pages_today: Vec<TopPageView>,
}

pub(super) struct ActivityData {
    pub hourly: Vec<HourlyBar>,
    pub skills: Vec<SkillBar>,
    pub tools: Vec<ToolSuccessBar>,
    // JSON: required by trait contract
    pub chart: serde_json::Value,
}

pub(super) fn build_activity_data(dash: &types::DashboardData, range_key: &str) -> ActivityData {
    let hourly_max = dash
        .hourly_activity
        .iter()
        .map(|h| h.count)
        .max()
        .unwrap_or(1)
        .max(1);
    let mut hours = [0i64; 24];
    for h in &dash.hourly_activity {
        if let Ok(idx) = usize::try_from(h.hour) {
            if idx < 24 {
                hours[idx] = h.count;
            }
        }
    }
    let hourly = hours
        .iter()
        .enumerate()
        .map(|(i, &count)| HourlyBar {
            hour: i,
            count,
            pct: count.saturating_mul(100) / hourly_max,
            label: if i % 3 == 0 {
                format!("{i}")
            } else {
                String::new()
            },
        })
        .collect();

    let skills_max = dash.popular_skills.first().map_or(1, |s| s.count).max(1);
    let skills = dash
        .popular_skills
        .iter()
        .map(|s| SkillBar {
            tool_name: s.tool_name.clone(),
            count: s.count,
            pct: s.count.saturating_mul(100) / skills_max,
        })
        .collect();

    let tools = dash
        .tool_success_rates
        .iter()
        .map(|t| {
            let color_class = if t.success_pct < 90.0 {
                "progress-red"
            } else if t.success_pct < 98.0 {
                "progress-amber"
            } else {
                "progress-green"
            };
            ToolSuccessBar {
                tool_name: t.tool_name.clone(),
                total: t.total,
                successes: t.successes,
                failures: t.failures,
                success_pct: format!("{:.1}", t.success_pct),
                pct: t.success_pct,
                color_class,
            }
        })
        .collect();

    // JSON: required by trait contract
    let chart = serde_json::to_value(super::charts::compute_area_chart_data(
        &dash.usage_timeseries,
        range_key,
    ))
    .unwrap_or_default();
    ActivityData {
        hourly,
        skills,
        tools,
        chart,
    }
}

pub(super) fn build_mcp_health(
    stats: &types::ActivityStats,
) -> (usize, &'static str, &'static str) {
    let mcp_error_rate_pct = if stats.mcp_tool_calls > 0 {
        stats.mcp_errors.saturating_mul(100) / stats.mcp_tool_calls
    } else {
        0
    };
    let (health_status, label) = if mcp_error_rate_pct == 0 {
        ("healthy", "Healthy")
    } else if mcp_error_rate_pct < 10 {
        ("warning", "Warning")
    } else {
        ("critical", "Critical")
    };
    (
        numeric::i64_to_usize(mcp_error_rate_pct),
        health_status,
        label,
    )
}
