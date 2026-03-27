use crate::admin::repositories::{profile_reports, session_analyses};
use serde_json::json;

pub(super) fn build_comparison_grid(
    user: &profile_reports::UserAggregateMetrics,
    global: &crate::admin::repositories::daily_summaries::GlobalAverages,
) -> Vec<serde_json::Value> {

    let row =
        |label: &str, user_val: f64, global_val: f64, positive_up: bool| -> serde_json::Value {
            let diff = user_val - global_val;
            let pct = if global_val > 0.0 {
                diff / global_val * 100.0
            } else {
                0.0
            };
            let effective_pct = if positive_up { pct } else { -pct };
            let sentiment = if effective_pct > 5.0 {
                "positive"
            } else if effective_pct < -5.0 {
                "negative"
            } else {
                "neutral"
            };
            let arrow = if effective_pct > 5.0 {
                "+"
            } else if effective_pct < -5.0 {
                "-"
            } else {
                "="
            };
            json!({
                "label": label,
                "user_value": format!("{user_val:.1}"),
                "global_value": format!("{global_val:.1}"),
                "diff_pct": format!("{:.0}%", pct.abs()),
                "sentiment": sentiment,
                "arrow": arrow,
                "bar_pct": bar_percentage(user_val, global_val),
                "global_bar_pct": bar_percentage(global_val, user_val),
            })
        };

    vec![
        row(
            "Quality Score",
            user.avg_quality,
            f64::from(global.avg_quality.unwrap_or(0.0)),
            true,
        ),
        row(
            "Actions/Min",
            user.avg_apm,
            f64::from(global.avg_apm.unwrap_or(0.0)),
            true,
        ),
        row(
            "Goal Rate %",
            user.avg_goal_rate,
            global.avg_goal_rate.unwrap_or(0.0),
            true,
        ),
        row(
            "Error Rate %",
            user.avg_error_rate,
            global.avg_error_rate.unwrap_or(0.0),
            false,
        ),
        row(
            "Tool Diversity",
            user.avg_tool_diversity,
            f64::from(global.avg_tool_diversity.unwrap_or(0.0)),
            true,
        ),
        row(
            "Multitasking",
            user.avg_multitasking,
            f64::from(global.avg_multitasking.unwrap_or(0.0)),
            true,
        ),
        row(
            "Sessions/Day",
            user.avg_sessions_per_day,
            f64::from(global.avg_sessions.unwrap_or(0.0)),
            true,
        ),
    ]
}

fn bar_percentage(user_val: f64, global_val: f64) -> f64 {
    let max_val = user_val.max(global_val);
    if max_val > 0.0 {
        (user_val / max_val * 100.0).round().clamp(2.0, 100.0)
    } else {
        50.0
    }
}

pub(super) fn build_category_breakdown(
    analyses: &[session_analyses::SessionAnalysisRow],
) -> Vec<serde_json::Value> {

    if analyses.is_empty() {
        return Vec::new();
    }

    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for a in analyses {
        *counts.entry(a.category.as_str()).or_insert(0) += 1;
    }

    let total = f64::from(u32::try_from(analyses.len()).unwrap_or(u32::MAX));
    let mut entries: Vec<_> = counts.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));

    entries
        .into_iter()
        .map(|(cat, count)| {
            let pct = (f64::from(u32::try_from(count).unwrap_or(u32::MAX)) / total * 100.0).round();
            json!({
                "category": cat,
                "label": format_category(cat),
                "count": count,
                "pct": pct,
                "bar_width": pct.max(2.0),
            })
        })
        .collect()
}

fn format_category(cat: &str) -> &str {
    match cat {
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

pub(super) fn build_gamification_data(
    gam: Option<&crate::admin::types::UserGamificationProfile>,
) -> serde_json::Value {

    let Some(g) = gam else { return json!(null) };
    let xp_pct = if g.xp_to_next_rank > 0 {
        let current_xp_in_level = g.total_xp % g.xp_to_next_rank;
        (f64::from(u32::try_from(current_xp_in_level).unwrap_or(0))
            / f64::from(u32::try_from(g.xp_to_next_rank).unwrap_or(1))
            * 100.0)
            .round()
    } else {
        100.0
    };
    json!({
        "rank_level": g.rank_level,
        "rank_name": g.rank_name,
        "total_xp": g.total_xp,
        "xp_to_next": g.xp_to_next_rank,
        "xp_pct": xp_pct,
        "next_rank_name": g.next_rank_name,
        "current_streak": g.current_streak,
        "longest_streak": g.longest_streak,
        "achievements_count": g.achievements.len(),
        "rank_position": g.rank_position,
    })
}

pub(super) fn build_ai_report_data(
    report: Option<&profile_reports::ProfileReportRow>,
) -> serde_json::Value {

    let Some(r) = report else { return json!(null) };
    json!({
        "narrative": r.ai_narrative,
        "style_analysis": r.ai_style_analysis,
        "comparison": r.ai_comparison,
        "patterns": r.ai_patterns,
        "improvements": r.ai_improvements,
        "tips": r.ai_tips,
        "generated_at": r.generated_at.format("%B %d, %Y at %H:%M UTC").to_string(),
        "has_narrative": r.ai_narrative.is_some(),
        "has_style": r.ai_style_analysis.is_some(),
        "has_comparison": r.ai_comparison.is_some(),
        "has_patterns": r.ai_patterns.is_some(),
        "has_improvements": r.ai_improvements.is_some(),
        "has_tips": r.ai_tips.is_some(),
    })
}
