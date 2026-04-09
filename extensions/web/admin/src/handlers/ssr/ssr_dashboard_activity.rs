use crate::types;

use super::ssr_dashboard_types::{HourlyBar, SkillBar, ToolSuccessBar};

pub(super) struct ActivityData {
    pub hourly: Vec<HourlyBar>,
    pub skills: Vec<SkillBar>,
    pub tools: Vec<ToolSuccessBar>,
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

    let chart = serde_json::to_value(super::charts::compute_area_chart_data(
        &dash.usage_timeseries,
        range_key,
    ))
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to serialize area chart data");
        serde_json::Value::Null
    });
    ActivityData {
        hourly,
        skills,
        tools,
        chart,
    }
}

pub(super) const fn build_mcp_health(
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
        crate::numeric::i64_to_usize(mcp_error_rate_pct),
        health_status,
        label,
    )
}
