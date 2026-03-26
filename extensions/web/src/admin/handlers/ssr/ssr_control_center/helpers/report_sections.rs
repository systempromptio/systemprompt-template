use crate::admin::numeric;
use crate::admin::repositories;
use crate::admin::repositories::apm_metrics::TodayPerformanceSummary;
use crate::admin::repositories::daily_summaries::DailySummaryRow;

use super::super::types::{HistoryEntry, InsightsData, InsightsFlags, MetricRow};
use super::metrics::{avg_field, make_metric_row, MetricRowInput};

pub(in crate::admin) fn build_performance_section(
    perf: &TodayPerformanceSummary,
    yesterday: &DailySummaryRow,
    days_7: &[DailySummaryRow],
    days_14: &[DailySummaryRow],
    global: &repositories::daily_summaries::GlobalAverages,
) -> Vec<MetricRow> {
    vec![
        make_metric_row(&MetricRowInput {
            label: "Sessions",
            today_val: numeric::to_f64(perf.total_sessions),
            yesterday_val: Some(f64::from(yesterday.session_count)),
            avg_7d: avg_field(days_7, |d| f64::from(d.session_count)),
            avg_14d: avg_field(days_14, |d| f64::from(d.session_count)),
            global_avg: global.avg_sessions.map(f64::from),
            positive_when_up: true,
        }),
        make_metric_row(&MetricRowInput {
            label: "Actions",
            today_val: numeric::to_f64(perf.total_actions),
            yesterday_val: Some(numeric::to_f64(
                yesterday.total_prompts + yesterday.total_tool_uses,
            )),
            avg_7d: avg_field(days_7, |d| {
                numeric::to_f64(d.total_prompts + d.total_tool_uses)
            }),
            avg_14d: avg_field(days_14, |d| {
                numeric::to_f64(d.total_prompts + d.total_tool_uses)
            }),
            global_avg: None,
            positive_when_up: true,
        }),
        make_metric_row(&MetricRowInput {
            label: "Prompts",
            today_val: numeric::to_f64(perf.total_prompts),
            yesterday_val: Some(numeric::to_f64(yesterday.total_prompts)),
            avg_7d: avg_field(days_7, |d| numeric::to_f64(d.total_prompts)),
            avg_14d: avg_field(days_14, |d| numeric::to_f64(d.total_prompts)),
            global_avg: None,
            positive_when_up: true,
        }),
        make_metric_row(&MetricRowInput {
            label: "Tool Uses",
            today_val: numeric::to_f64(perf.total_tool_uses),
            yesterday_val: Some(numeric::to_f64(yesterday.total_tool_uses)),
            avg_7d: avg_field(days_7, |d| numeric::to_f64(d.total_tool_uses)),
            avg_14d: avg_field(days_14, |d| numeric::to_f64(d.total_tool_uses)),
            global_avg: None,
            positive_when_up: true,
        }),
        make_metric_row(&MetricRowInput {
            label: "Errors",
            today_val: numeric::to_f64(perf.total_errors),
            yesterday_val: Some(numeric::to_f64(yesterday.total_errors)),
            avg_7d: avg_field(days_7, |d| numeric::to_f64(d.total_errors)),
            avg_14d: avg_field(days_14, |d| numeric::to_f64(d.total_errors)),
            global_avg: None,
            positive_when_up: false,
        }),
        build_error_rate_row(perf, yesterday, days_7, days_14, global),
        make_metric_row(&MetricRowInput {
            label: "Active Minutes",
            today_val: f64::from(perf.active_minutes),
            yesterday_val: None,
            avg_7d: None,
            avg_14d: None,
            global_avg: None,
            positive_when_up: true,
        }),
    ]
}

pub(in crate::admin) fn build_productivity_section(
    perf: &TodayPerformanceSummary,
    yesterday: &DailySummaryRow,
    days_7: &[DailySummaryRow],
    days_14: &[DailySummaryRow],
    global: &repositories::daily_summaries::GlobalAverages,
) -> Vec<MetricRow> {
    vec![
        make_metric_row(&MetricRowInput {
            label: "Avg APM",
            today_val: f64::from(perf.avg_apm),
            yesterday_val: yesterday.avg_apm.map(f64::from),
            avg_7d: avg_field(days_7, |d| f64::from(d.avg_apm.unwrap_or(0.0))),
            avg_14d: avg_field(days_14, |d| f64::from(d.avg_apm.unwrap_or(0.0))),
            global_avg: global.avg_apm.map(f64::from),
            positive_when_up: true,
        }),
        make_metric_row(&MetricRowInput {
            label: "Peak APM",
            today_val: f64::from(perf.peak_apm),
            yesterday_val: yesterday.peak_apm.map(f64::from),
            avg_7d: avg_field(days_7, |d| f64::from(d.peak_apm.unwrap_or(0.0))),
            avg_14d: avg_field(days_14, |d| f64::from(d.peak_apm.unwrap_or(0.0))),
            global_avg: global.avg_peak_apm.map(f64::from),
            positive_when_up: true,
        }),
        make_metric_row(&MetricRowInput {
            label: "Tool Diversity",
            today_val: f64::from(perf.tool_diversity),
            yesterday_val: Some(f64::from(yesterday.tool_diversity)),
            avg_7d: avg_field(days_7, |d| f64::from(d.tool_diversity)),
            avg_14d: avg_field(days_14, |d| f64::from(d.tool_diversity)),
            global_avg: global.avg_tool_diversity.map(f64::from),
            positive_when_up: true,
        }),
        make_metric_row(&MetricRowInput {
            label: "Multitasking",
            today_val: f64::from(perf.multitasking_score),
            yesterday_val: yesterday.multitasking_score.map(f64::from),
            avg_7d: avg_field(days_7, |d| f64::from(d.multitasking_score.unwrap_or(0.0))),
            avg_14d: avg_field(days_14, |d| f64::from(d.multitasking_score.unwrap_or(0.0))),
            global_avg: global.avg_multitasking.map(f64::from),
            positive_when_up: true,
        }),
        make_metric_row(&MetricRowInput {
            label: "Throughput",
            today_val: numeric::to_f64(perf.total_input_bytes + perf.total_output_bytes),
            yesterday_val: Some(numeric::to_f64(
                yesterday.total_input_bytes + yesterday.total_output_bytes,
            )),
            avg_7d: avg_field(days_7, |d| {
                numeric::to_f64(d.total_input_bytes + d.total_output_bytes)
            }),
            avg_14d: avg_field(days_14, |d| {
                numeric::to_f64(d.total_input_bytes + d.total_output_bytes)
            }),
            global_avg: global.avg_throughput.map(numeric::to_f64),
            positive_when_up: true,
        }),
        make_metric_row(&MetricRowInput {
            label: "Session Velocity",
            today_val: 0.0,
            yesterday_val: yesterday.session_velocity.map(f64::from),
            avg_7d: avg_field(days_7, |d| f64::from(d.session_velocity.unwrap_or(0.0))),
            avg_14d: avg_field(days_14, |d| f64::from(d.session_velocity.unwrap_or(0.0))),
            global_avg: None,
            positive_when_up: true,
        }),
    ]
}

pub(in crate::admin) fn build_insights(yesterday: &DailySummaryRow) -> InsightsData {
    InsightsData {
        summary: yesterday.summary.clone(),
        patterns: yesterday.patterns.as_deref().unwrap_or("").to_string(),
        skill_gaps: yesterday.skill_gaps.as_deref().unwrap_or("").to_string(),
        top_recommendation: yesterday
            .top_recommendation
            .as_deref()
            .unwrap_or("")
            .to_string(),
        highlights: yesterday.highlights.as_deref().unwrap_or("").to_string(),
        trends: yesterday.trends.as_deref().unwrap_or("").to_string(),
        flags: InsightsFlags {
            has_patterns: yesterday.patterns.is_some(),
            has_skill_gaps: yesterday.skill_gaps.is_some(),
            has_recommendation: yesterday.top_recommendation.is_some(),
            has_highlights: yesterday.highlights.is_some(),
            has_trends: yesterday.trends.is_some(),
        },
    }
}

pub(in crate::admin) fn build_history(daily_summaries: &[DailySummaryRow]) -> Vec<HistoryEntry> {
    daily_summaries
        .iter()
        .take(7)
        .rev()
        .map(|d| HistoryEntry {
            date: d.summary_date.format("%m/%d").to_string(),
            sessions: d.session_count,
            quality: d.avg_quality_score.unwrap_or(0.0),
            apm: d.avg_apm.unwrap_or(0.0),
            errors: d.total_errors,
        })
        .collect()
}

fn error_rate_for_row(d: &DailySummaryRow) -> f64 {
    let a = numeric::to_f64(d.total_prompts + d.total_tool_uses);
    if a > 0.0 {
        numeric::to_f64(d.total_errors) / a * 100.0
    } else {
        0.0
    }
}

fn build_error_rate_row(
    perf: &TodayPerformanceSummary,
    yesterday: &DailySummaryRow,
    days_7: &[DailySummaryRow],
    days_14: &[DailySummaryRow],
    global: &repositories::daily_summaries::GlobalAverages,
) -> MetricRow {
    let yesterday_actions = numeric::to_f64(yesterday.total_prompts + yesterday.total_tool_uses);
    let yesterday_error_rate = if yesterday_actions > 0.0 {
        Some(numeric::to_f64(yesterday.total_errors) / yesterday_actions * 100.0)
    } else {
        Some(0.0)
    };
    make_metric_row(&MetricRowInput {
        label: "Error Rate",
        today_val: f64::from(perf.error_rate_pct),
        yesterday_val: yesterday_error_rate,
        avg_7d: avg_field(days_7, error_rate_for_row),
        avg_14d: avg_field(days_14, error_rate_for_row),
        global_avg: global.avg_error_rate,
        positive_when_up: false,
    })
}
