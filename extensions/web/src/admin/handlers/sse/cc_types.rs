use serde::Serialize;

use crate::admin::numeric;
use crate::admin::repositories::{apm_metrics, session_analyses};

#[derive(Serialize)]
pub(super) struct EntityLinkEntry<'a> {
    pub session_id: &'a str,
    pub entity_type: &'a str,
    pub entity_name: &'a str,
    pub usage_count: i32,
}

#[derive(Serialize)]
pub(super) struct HealthEntry {
    pub score: i64,
    pub label: &'static str,
    pub color_class: &'static str,
}

#[derive(Serialize)]
pub(super) struct ApmCorrelationEntry {
    pub high_apm_success_rate: f32,
    pub low_apm_success_rate: f32,
}

#[derive(Serialize)]
pub(super) struct TodaySummaryEntry {
    pub sessions_count: i64,
    pub analysed_count: i64,
    pub avg_quality: String,
    pub goals_achieved: i64,
    pub goals_partial: i64,
    pub goals_failed: i64,
    pub new_achievements: Vec<String>,
    pub has_new_achievements: bool,
    pub top_recommendation: String,
    pub has_top_recommendation: bool,
    pub avg_apm: String,
    pub peak_apm: String,
    pub session_velocity: String,
    pub achievements_today: Vec<String>,
    pub apm_correlation: ApmCorrelationEntry,
}

#[derive(Serialize)]
pub(super) struct HourlyEntry {
    pub hour: i32,
    pub actions: i64,
    pub errors: i64,
    pub sessions: i64,
    pub input_bytes: i64,
    pub output_bytes: i64,
    pub unique_tools: i64,
    pub subagent_spawns: i64,
}

#[derive(Serialize)]
pub(super) struct PerformanceEntry {
    pub total_sessions: i64,
    pub total_actions: i64,
    pub total_prompts: i64,
    pub total_tool_uses: i64,
    pub total_errors: i64,
    pub error_rate_pct: String,
    pub total_input_bytes: i64,
    pub total_output_bytes: i64,
    pub avg_apm: String,
    pub peak_apm: String,
    pub peak_concurrency: i32,
    pub tool_diversity: i32,
    pub multitasking_score: String,
    pub active_minutes: String,
}

#[derive(Serialize)]
pub(super) struct AchievementProgressEntry {
    pub id: &'static str,
    pub name: &'static str,
    pub current: i64,
    pub threshold: i64,
    pub remaining: i64,
    pub pct: i64,
}

pub(super) fn build_health(h: &session_analyses::HealthMetrics) -> Option<HealthEntry> {
    if h.total_sessions_30d == 0 {
        return None;
    }
    let (label, color_class) = match h.health_score {
        90..=100 => ("Excellent", "cc-health-excellent"),
        75..=89 => ("Good", "cc-health-good"),
        50..=74 => ("Fair", "cc-health-fair"),
        _ => ("Needs Attention", "cc-health-attention"),
    };
    Some(HealthEntry {
        score: h.health_score,
        label,
        color_class,
    })
}

pub(super) fn build_today_summary(
    ts: &session_analyses::TodaySummary,
    apm_live: &apm_metrics::TodayApmLive,
    apm_correlation: &apm_metrics::ApmCorrelation,
    achievements_today: &[String],
) -> TodaySummaryEntry {
    TodaySummaryEntry {
        sessions_count: ts.sessions_count,
        analysed_count: ts.analysed_count,
        avg_quality: format!("{:.1}", ts.avg_quality),
        goals_achieved: ts.goals_achieved,
        goals_partial: ts.goals_partial,
        goals_failed: ts.goals_failed,
        has_new_achievements: !ts.new_achievements.is_empty(),
        new_achievements: ts.new_achievements.clone(),
        has_top_recommendation: !ts.top_recommendation.is_empty(),
        top_recommendation: ts.top_recommendation.clone(),
        avg_apm: format!("{:.1}", apm_live.avg_apm),
        peak_apm: format!("{:.1}", apm_live.peak_apm),
        session_velocity: format!("{:.1}", apm_live.multitasking_score),
        achievements_today: achievements_today.to_vec(),
        apm_correlation: ApmCorrelationEntry {
            high_apm_success_rate: apm_correlation.high_apm_success_rate,
            low_apm_success_rate: apm_correlation.low_apm_success_rate,
        },
    }
}

pub(super) fn build_hourly(hourly: &[apm_metrics::HourlyApmBucket]) -> Vec<HourlyEntry> {
    hourly
        .iter()
        .map(|b| HourlyEntry {
            hour: b.hour,
            actions: b.actions,
            errors: b.errors,
            sessions: b.sessions,
            input_bytes: b.input_bytes,
            output_bytes: b.output_bytes,
            unique_tools: b.unique_tools,
            subagent_spawns: b.subagent_spawns,
        })
        .collect()
}

#[derive(Serialize)]
pub(super) struct GamificationBlock {
    pub has_gamification: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_xp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xp_to_next_rank: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_rank_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_streak: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub achievements_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub achievements_total: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xp_progress_pct: Option<i64>,
}

pub(super) fn build_gamification_block(
    gam: Option<&crate::admin::types::UserGamificationProfile>,
) -> GamificationBlock {
    if let Some(g) = gam {
        let total_for_rank = g.total_xp + g.xp_to_next_rank;
        let xp_pct = if total_for_rank > 0 {
            numeric::pct_i64(g.total_xp, total_for_rank)
        } else {
            100
        };
        GamificationBlock {
            has_gamification: true,
            rank_level: Some(g.rank_level),
            rank_name: Some(g.rank_name.clone()),
            total_xp: Some(g.total_xp),
            xp_to_next_rank: Some(g.xp_to_next_rank),
            next_rank_name: Some(
                g.next_rank_name
                    .as_deref()
                    .unwrap_or("Max Rank")
                    .to_string(),
            ),
            current_streak: Some(g.current_streak),
            achievements_count: Some(g.achievements.len()),
            achievements_total: Some(60),
            xp_progress_pct: Some(xp_pct),
        }
    } else {
        GamificationBlock {
            has_gamification: false,
            rank_level: None,
            rank_name: None,
            total_xp: None,
            xp_to_next_rank: None,
            next_rank_name: None,
            current_streak: None,
            achievements_count: None,
            achievements_total: None,
            xp_progress_pct: None,
        }
    }
}

pub(super) fn build_performance(p: &apm_metrics::TodayPerformanceSummary) -> PerformanceEntry {
    PerformanceEntry {
        total_sessions: p.total_sessions,
        total_actions: p.total_actions,
        total_prompts: p.total_prompts,
        total_tool_uses: p.total_tool_uses,
        total_errors: p.total_errors,
        error_rate_pct: format!("{:.1}", p.error_rate_pct),
        total_input_bytes: p.total_input_bytes,
        total_output_bytes: p.total_output_bytes,
        avg_apm: format!("{:.1}", p.avg_apm),
        peak_apm: format!("{:.1}", p.peak_apm),
        peak_concurrency: p.peak_concurrency,
        tool_diversity: p.tool_diversity,
        multitasking_score: format!("{:.1}", p.multitasking_score),
        active_minutes: format!("{:.0}", p.active_minutes),
    }
}
