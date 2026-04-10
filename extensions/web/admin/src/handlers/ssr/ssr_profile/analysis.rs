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
    let deviations = collect_all_deviations(user, global);
    partition_and_sort(deviations)
}

fn collect_all_deviations(
    user: &UserAggregateMetrics,
    global: &GlobalAverages,
) -> Vec<MetricDeviation> {
    let mut deviations = Vec::new();
    let entries = build_deviation_entries(user, global);

    for entry in &entries {
        push_deviation(&mut deviations, entry);
    }

    deviations
}

fn build_deviation_entries<'a>(
    user: &'a UserAggregateMetrics,
    global: &'a GlobalAverages,
) -> Vec<DeviationInput<'a>> {
    vec![
        DeviationInput {
            metric: "quality",
            label: "Quality Score",
            user_val: user.avg_quality,
            global_val: f64::from(global.avg_quality.unwrap_or(0.0)),
            positive_up: true,
            format_fn: fmt_decimal,
        },
        DeviationInput {
            metric: "apm",
            label: "Actions Per Minute",
            user_val: user.avg_apm,
            global_val: f64::from(global.avg_apm.unwrap_or(0.0)),
            positive_up: true,
            format_fn: fmt_decimal,
        },
        DeviationInput {
            metric: "goal_rate",
            label: "Goal Achievement Rate",
            user_val: user.avg_goal_rate,
            global_val: global.avg_goal_rate.unwrap_or(0.0),
            positive_up: true,
            format_fn: fmt_percent,
        },
        DeviationInput {
            metric: "error_rate",
            label: "Error Rate",
            user_val: user.avg_error_rate,
            global_val: global.avg_error_rate.unwrap_or(0.0),
            positive_up: false,
            format_fn: fmt_percent,
        },
        DeviationInput {
            metric: "tool_diversity",
            label: "Tool Diversity",
            user_val: user.avg_tool_diversity,
            global_val: f64::from(global.avg_tool_diversity.unwrap_or(0.0)),
            positive_up: true,
            format_fn: fmt_whole,
        },
        DeviationInput {
            metric: "multitasking",
            label: "Multitasking Score",
            user_val: user.avg_multitasking,
            global_val: f64::from(global.avg_multitasking.unwrap_or(0.0)),
            positive_up: true,
            format_fn: fmt_decimal,
        },
        DeviationInput {
            metric: "throughput",
            label: "Data Throughput",
            user_val: user.avg_throughput,
            global_val: global
                .avg_throughput
                .map_or(0.0, |v| f64::from(u32::try_from(v).unwrap_or(0))),
            positive_up: true,
            format_fn: fmt_bytes,
        },
        DeviationInput {
            metric: "sessions_per_day",
            label: "Sessions Per Day",
            user_val: user.avg_sessions_per_day,
            global_val: f64::from(global.avg_sessions.unwrap_or(0.0)),
            positive_up: true,
            format_fn: fmt_decimal,
        },
    ]
}

fn partition_and_sort(
    deviations: Vec<MetricDeviation>,
) -> (Vec<MetricDeviation>, Vec<MetricDeviation>) {
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

struct DeviationInput<'a> {
    metric: &'a str,
    label: &'a str,
    user_val: f64,
    global_val: f64,
    positive_up: bool,
    format_fn: fn(f64) -> String,
}

fn push_deviation(devs: &mut Vec<MetricDeviation>, input: &DeviationInput<'_>) {
    if input.global_val <= 0.0 && input.user_val <= 0.0 {
        return;
    }
    let raw_pct = if input.global_val > 0.0 {
        (input.user_val - input.global_val) / input.global_val * 100.0
    } else if input.user_val > 0.0 {
        100.0
    } else {
        0.0
    };
    let effective_dev = if input.positive_up { raw_pct } else { -raw_pct };
    devs.push(MetricDeviation {
        metric: input.metric.into(),
        label: input.label.into(),
        user_value: input.user_val,
        global_value: input.global_val,
        deviation_pct: effective_dev.round(),
        formatted_user: (input.format_fn)(input.user_val),
        formatted_global: (input.format_fn)(input.global_val),
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
