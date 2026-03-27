use crate::admin::repositories::daily_summaries::{DailySummaryRow, GlobalAverages};
use serde_json::json;

pub fn build_trend_data(
    daily_summaries: &[DailySummaryRow],
    global: &GlobalAverages,
) -> serde_json::Value {
    if daily_summaries.is_empty() {
        return json!({ "has_trends": false });
    }

    let days: Vec<&DailySummaryRow> = daily_summaries.iter().take(30).rev().collect();

    let dates: Vec<String> = days
        .iter()
        .map(|d| d.summary_date.format("%m/%d").to_string())
        .collect();
    let quality: Vec<f64> = days
        .iter()
        .map(|d| f64::from(d.avg_quality_score.unwrap_or(0.0)))
        .collect();
    let apm: Vec<f64> = days
        .iter()
        .map(|d| f64::from(d.avg_apm.unwrap_or(0.0)))
        .collect();
    let sessions: Vec<f64> = days.iter().map(|d| f64::from(d.session_count)).collect();
    let error_rates: Vec<f64> = days.iter().map(|d| compute_error_rate(d)).collect();
    let goal_rates: Vec<f64> = days.iter().map(|d| compute_goal_rate(d)).collect();
    let tool_div: Vec<f64> = days.iter().map(|d| f64::from(d.tool_diversity)).collect();

    json!({
        "has_trends": true,
        "dates": dates,
        "quality": { "values": csv(&quality), "global_avg": global.avg_quality.unwrap_or(0.0) },
        "apm": { "values": csv(&apm), "global_avg": global.avg_apm.unwrap_or(0.0) },
        "sessions": { "values": csv(&sessions), "global_avg": global.avg_sessions.unwrap_or(0.0) },
        "error_rate": { "values": csv(&error_rates), "global_avg": global.avg_error_rate.unwrap_or(0.0) },
        "goal_rate": { "values": csv(&goal_rates), "global_avg": global.avg_goal_rate.unwrap_or(0.0) },
        "tool_diversity": { "values": csv(&tool_div), "global_avg": global.avg_tool_diversity.unwrap_or(0.0) },
    })
}

fn compute_error_rate(d: &DailySummaryRow) -> f64 {
    let total = d.total_prompts + d.total_tool_uses;
    if total > 0 {
        f64::from(u32::try_from(d.total_errors).unwrap_or(0))
            / f64::from(u32::try_from(total).unwrap_or(1))
            * 100.0
    } else {
        0.0
    }
}

fn compute_goal_rate(d: &DailySummaryRow) -> f64 {
    let total = d.goals_achieved + d.goals_failed;
    if total > 0 {
        f64::from(d.goals_achieved) / f64::from(total) * 100.0
    } else {
        0.0
    }
}

fn csv(values: &[f64]) -> String {
    values
        .iter()
        .map(|v| format!("{v:.1}"))
        .collect::<Vec<_>>()
        .join(",")
}
