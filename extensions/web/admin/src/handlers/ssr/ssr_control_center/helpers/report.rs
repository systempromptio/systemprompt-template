use std::collections::HashMap;

use crate::numeric;
use crate::repositories;
use crate::repositories::apm_metrics::TodayPerformanceSummary;
use crate::repositories::daily_summaries::DailySummaryRow;
use crate::repositories::session_analyses::{
    HealthMetrics, SessionAnalysisRow, TodaySummary,
};

use super::super::types::{CategoryBreakdownEntry, EntityCounts, ReportData};
use super::metrics::{avg_field, make_metric_row, MetricRowInput};
use super::report_sections;

pub(in crate::admin) struct ReportParams<'a> {
    pub perf: &'a TodayPerformanceSummary,
    pub today: &'a TodaySummary,
    pub health: &'a HealthMetrics,
    pub daily_summaries: &'a [DailySummaryRow],
    pub global: &'a repositories::daily_summaries::GlobalAverages,
    pub entity_counts: &'a EntityCounts,
    pub current_streak: i32,
    pub recent_analyses: &'a [SessionAnalysisRow],
}

pub(in crate::admin) fn build_report_data(params: &ReportParams<'_>) -> ReportData {
    if params.daily_summaries.is_empty() {
        return ReportData {
            has_data: false,
            report_date: chrono::Local::now().format("%B %d, %Y").to_string(),
            streak: None,
            performance: None,
            quality: None,
            productivity: None,
            insights: None,
            history: None,
            entity_counts: None,
            category_breakdown: None,
            has_category_breakdown: None,
        };
    }

    let yesterday = &params.daily_summaries[0];
    let days_7 = &params.daily_summaries[..params.daily_summaries.len().min(7)];
    let days_14 = &params.daily_summaries[..params.daily_summaries.len().min(14)];

    let performance = report_sections::build_performance_section(
        params.perf,
        yesterday,
        days_7,
        days_14,
        params.global,
    );
    let quality = build_quality_section(
        params.today,
        params.health,
        yesterday,
        days_7,
        days_14,
        params.global,
    );
    let productivity = report_sections::build_productivity_section(
        params.perf,
        yesterday,
        days_7,
        days_14,
        params.global,
    );
    let insights = report_sections::build_insights(yesterday);
    let history = report_sections::build_history(params.daily_summaries);
    let category_breakdown = build_category_breakdown(params.recent_analyses);
    let has_category_breakdown = !category_breakdown.is_empty();

    ReportData {
        has_data: true,
        report_date: chrono::Local::now().format("%B %d, %Y").to_string(),
        streak: Some(params.current_streak),
        performance: Some(performance),
        quality: Some(quality),
        productivity: Some(productivity),
        insights: Some(insights),
        history: Some(history),
        entity_counts: Some(*params.entity_counts),
        category_breakdown: Some(category_breakdown),
        has_category_breakdown: Some(has_category_breakdown),
    }
}

fn build_quality_section(
    today: &TodaySummary,
    health: &HealthMetrics,
    yesterday: &DailySummaryRow,
    days_7: &[DailySummaryRow],
    days_14: &[DailySummaryRow],
    global: &repositories::daily_summaries::GlobalAverages,
) -> Vec<super::super::types::MetricRow> {
    vec![
        make_metric_row(&MetricRowInput {
            label: "Avg Quality",
            today_val: today.avg_quality,
            yesterday_val: yesterday.avg_quality_score.map(f64::from),
            avg_7d: avg_field(days_7, |d| f64::from(d.avg_quality_score.unwrap_or(0.0))),
            avg_14d: avg_field(days_14, |d| f64::from(d.avg_quality_score.unwrap_or(0.0))),
            global_avg: global.avg_quality.map(f64::from),
            positive_when_up: true,
        }),
        make_metric_row(&MetricRowInput {
            label: "Goals Achieved",
            today_val: numeric::to_f64(today.goals_achieved),
            yesterday_val: Some(f64::from(yesterday.goals_achieved)),
            avg_7d: avg_field(days_7, |d| f64::from(d.goals_achieved)),
            avg_14d: avg_field(days_14, |d| f64::from(d.goals_achieved)),
            global_avg: None,
            positive_when_up: true,
        }),
        make_metric_row(&MetricRowInput {
            label: "Goals Failed",
            today_val: numeric::to_f64(today.goals_failed),
            yesterday_val: Some(f64::from(yesterday.goals_failed)),
            avg_7d: avg_field(days_7, |d| f64::from(d.goals_failed)),
            avg_14d: avg_field(days_14, |d| f64::from(d.goals_failed)),
            global_avg: None,
            positive_when_up: false,
        }),
        make_metric_row(&MetricRowInput {
            label: "Health Score",
            today_val: numeric::to_f64(health.health_score),
            yesterday_val: None,
            avg_7d: None,
            avg_14d: None,
            global_avg: None,
            positive_when_up: true,
        }),
    ]
}

fn build_category_breakdown(analyses: &[SessionAnalysisRow]) -> Vec<CategoryBreakdownEntry> {
    if analyses.is_empty() {
        return Vec::new();
    }

    let mut counts: HashMap<&str, usize> = HashMap::new();
    for a in analyses {
        *counts.entry(a.category.as_str()).or_insert(0) += 1;
    }

    let total = numeric::usize_to_f64(analyses.len());
    let mut entries: Vec<_> = counts.into_iter().collect();
    entries.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    entries
        .into_iter()
        .map(|(category, count)| {
            let pct = (numeric::usize_to_f64(count) / total * 100.0).round();
            let label = format_category_label(category);
            CategoryBreakdownEntry {
                category: category.to_string(),
                label,
                count,
                pct,
                bar_width: pct.max(2.0),
            }
        })
        .collect()
}

fn format_category_label(category: &str) -> &'static str {
    match category {
        "feature" => "Feature",
        "bugfix" => "Bug Fix",
        "refactoring" => "Refactoring",
        "techdebt" => "Tech Debt",
        "documentation" => "Documentation",
        "discovery" => "Discovery",
        "testing" => "Testing",
        "deployment" => "Deployment",
        "configuration" => "Configuration",
        "design" => "Design",
        "review" => "Review",
        _ => "Other",
    }
}
