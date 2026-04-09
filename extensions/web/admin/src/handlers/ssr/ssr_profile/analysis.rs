use serde::Serialize;

use crate::repositories::daily_summaries::GlobalAverages;
use crate::repositories::profile_reports::UserAggregateMetrics;

const STRENGTH_THRESHOLD: f64 = 15.0;
const WEAKNESS_THRESHOLD: f64 = -15.0;

#[derive(Debug, Serialize)]
pub struct MetricDeviation {
    pub metric: String,
    pub label: String,
    pub user_value: f64,
    pub global_value: f64,
    pub deviation_pct: f64,
    pub formatted_user: String,
    pub formatted_global: String,
}

pub fn compute_strengths_weaknesses(
    user: &UserAggregateMetrics,
    global: &GlobalAverages,
) -> (Vec<MetricDeviation>, Vec<MetricDeviation>) {
    let mut deviations = Vec::new();

    push_deviation(
        &mut deviations,
        "quality",
        "Quality Score",
        user.avg_quality,
        f64::from(global.avg_quality.unwrap_or(0.0)),
        true,
        fmt_decimal,
    );
    push_deviation(
        &mut deviations,
        "apm",
        "Actions Per Minute",
        user.avg_apm,
        f64::from(global.avg_apm.unwrap_or(0.0)),
        true,
        fmt_decimal,
    );
    push_deviation(
        &mut deviations,
        "goal_rate",
        "Goal Achievement Rate",
        user.avg_goal_rate,
        global.avg_goal_rate.unwrap_or(0.0),
        true,
        fmt_percent,
    );
    push_deviation(
        &mut deviations,
        "error_rate",
        "Error Rate",
        user.avg_error_rate,
        global.avg_error_rate.unwrap_or(0.0),
        false,
        fmt_percent,
    );
    push_deviation(
        &mut deviations,
        "tool_diversity",
        "Tool Diversity",
        user.avg_tool_diversity,
        f64::from(global.avg_tool_diversity.unwrap_or(0.0)),
        true,
        fmt_whole,
    );
    push_deviation(
        &mut deviations,
        "multitasking",
        "Multitasking Score",
        user.avg_multitasking,
        f64::from(global.avg_multitasking.unwrap_or(0.0)),
        true,
        fmt_decimal,
    );
    push_deviation(
        &mut deviations,
        "throughput",
        "Data Throughput",
        user.avg_throughput,
        global
            .avg_throughput
            .map_or(0.0, |v| f64::from(u32::try_from(v).unwrap_or(0))),
        true,
        fmt_bytes,
    );
    push_deviation(
        &mut deviations,
        "sessions_per_day",
        "Sessions Per Day",
        user.avg_sessions_per_day,
        f64::from(global.avg_sessions.unwrap_or(0.0)),
        true,
        fmt_decimal,
    );

    let mut strengths = Vec::new();
    let mut weaknesses = Vec::new();

    for dev in deviations {
        if dev.deviation_pct >= STRENGTH_THRESHOLD {
            strengths.push(dev);
        } else if dev.deviation_pct <= WEAKNESS_THRESHOLD {
            weaknesses.push(dev);
        }
    }

    strengths.sort_unstable_by(|a, b| {
        b.deviation_pct
            .partial_cmp(&a.deviation_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    weaknesses.sort_unstable_by(|a, b| {
        a.deviation_pct
            .partial_cmp(&b.deviation_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    (strengths, weaknesses)
}

fn push_deviation(
    devs: &mut Vec<MetricDeviation>,
    metric: &str,
    label: &str,
    user_val: f64,
    global_val: f64,
    positive_up: bool,
    format_fn: fn(f64) -> String,
) {
    if global_val <= 0.0 && user_val <= 0.0 {
        return;
    }
    let raw_pct = if global_val > 0.0 {
        (user_val - global_val) / global_val * 100.0
    } else if user_val > 0.0 {
        100.0
    } else {
        0.0
    };
    let effective_dev = if positive_up { raw_pct } else { -raw_pct };
    devs.push(MetricDeviation {
        metric: metric.into(),
        label: label.into(),
        user_value: user_val,
        global_value: global_val,
        deviation_pct: effective_dev.round(),
        formatted_user: format_fn(user_val),
        formatted_global: format_fn(global_val),
    });
}

fn fmt_decimal(v: f64) -> String {
    format!("{v:.1}")
}
fn fmt_percent(v: f64) -> String {
    format!("{v:.1}%")
}
fn fmt_whole(v: f64) -> String {
    format!("{v:.0}")
}

fn fmt_bytes(v: f64) -> String {
    if v > 1_000_000.0 {
        format!("{:.1}MB", v / 1_000_000.0)
    } else if v > 1_000.0 {
        format!("{:.0}KB", v / 1_000.0)
    } else {
        format!("{v:.0}B")
    }
}
