use crate::numeric;
use crate::repositories::session_analyses::HealthMetrics;
use crate::types::UserGamificationProfile;

use super::super::types::{AchievementProgress, HealthObj, SessionGroup};

pub(crate) struct SessionCounts {
    pub active: usize,
    pub analysed_closed: usize,
    pub has_any_data: bool,
}

pub(crate) fn build_session_counts(session_groups: &[SessionGroup]) -> SessionCounts {
    let active = session_groups.iter().filter(|g| g.flags.is_active).count();
    let analysed_closed = session_groups
        .iter()
        .filter(|g| g.flags.is_analysed && !g.flags.is_active)
        .count();

    SessionCounts {
        active,
        analysed_closed,
        has_any_data: !session_groups.is_empty(),
    }
}

pub(crate) struct GamificationData {
    pub has_gamification: bool,
    pub rank_level: i32,
    pub rank_name: String,
    pub total_xp: i64,
    pub xp_to_next: i64,
    pub next_rank_name: String,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub achievements_count: usize,
    pub achievements_total: usize,
    pub xp_progress_pct: i64,
    pub achievement_progress: Vec<AchievementProgress>,
}

pub(crate) fn build_gamification_data(
    gamification: Option<&UserGamificationProfile>,
    skills_usage: &[&crate::types::conversation_analytics::EntityUsageSummary],
    mcp_usage: &[&crate::types::conversation_analytics::EntityUsageSummary],
) -> GamificationData {
    let has_gamification = gamification.is_some();
    let (
        rank_level,
        rank_name,
        total_xp,
        xp_to_next,
        next_rank_name,
        current_streak,
        longest_streak,
        achievements_count,
        achievements_total,
        xp_progress_pct,
    ) = if let Some(g) = gamification {
        let total_for_rank = g.total_xp + g.xp_to_next_rank;
        let pct = if total_for_rank > 0 {
            let p = numeric::to_i64(
                numeric::to_f64(g.total_xp) / numeric::to_f64(total_for_rank) * 100.0,
            );
            p.min(100)
        } else {
            100
        };
        (
            g.rank_level,
            g.rank_name.as_str(),
            g.total_xp,
            g.xp_to_next_rank,
            g.next_rank_name.as_deref().unwrap_or("Max Rank"),
            g.current_streak,
            g.longest_streak,
            g.achievements.len(),
            60_usize,
            pct,
        )
    } else {
        (0, "", 0, 0, "", 0, 0, 0, 60, 0)
    };

    let achievement_progress = build_achievement_progress(gamification, skills_usage, mcp_usage);

    GamificationData {
        has_gamification,
        rank_level,
        rank_name: rank_name.to_string(),
        total_xp,
        xp_to_next,
        next_rank_name: next_rank_name.to_string(),
        current_streak,
        longest_streak,
        achievements_count,
        achievements_total,
        xp_progress_pct,
        achievement_progress,
    }
}

fn build_achievement_progress(
    gamification: Option<&UserGamificationProfile>,
    skills_usage: &[&crate::types::conversation_analytics::EntityUsageSummary],
    mcp_usage: &[&crate::types::conversation_analytics::EntityUsageSummary],
) -> Vec<AchievementProgress> {
    let mut achievement_progress = Vec::new();
    if let Some(g) = gamification {
        let unlocked_ids: std::collections::HashSet<&str> = g
            .achievements
            .iter()
            .map(|a| a.achievement_id.as_str())
            .collect();

        let skill_total: i64 = skills_usage.iter().map(|s| s.total_uses).sum();
        let unique_skills = numeric::usize_to_i64(skills_usage.len());
        let mcp_total: i64 = mcp_usage.iter().map(|m| m.total_uses).sum();
        let unique_mcp = numeric::usize_to_i64(mcp_usage.len());

        let milestones: Vec<(&str, &str, i64, i64)> = vec![
            ("skill_use_1", "Skill Invoker", skill_total, 1),
            ("skill_use_25", "Skill Enthusiast", skill_total, 25),
            ("skill_use_100", "Skill Virtuoso", skill_total, 100),
            ("skill_unique_5", "Skill Explorer", unique_skills, 5),
            ("skill_unique_10", "Skill Polymath", unique_skills, 10),
            ("mcp_use_1", "Server Link", mcp_total, 1),
            ("mcp_use_25", "Server Regular", mcp_total, 25),
            ("mcp_use_100", "Server Power User", mcp_total, 100),
            ("mcp_use_500", "Server Master", mcp_total, 500),
            ("mcp_unique_3", "Server Collector", unique_mcp, 3),
            ("mcp_unique_5", "Server Network", unique_mcp, 5),
        ];

        for (id, name, current, threshold) in milestones {
            if !unlocked_ids.contains(id) {
                let remaining = (threshold - current).max(0);
                let pct = if threshold > 0 {
                    ((current * 100) / threshold).min(100)
                } else {
                    0
                };
                achievement_progress.push(AchievementProgress {
                    id,
                    name,
                    current,
                    threshold,
                    remaining,
                    pct,
                });
            }
        }
    }
    achievement_progress
}

pub(crate) struct HealthData {
    pub has_health: bool,
    pub health_obj: HealthObj,
}

pub(crate) fn build_health_data(health: &HealthMetrics) -> HealthData {
    let health_label = match health.health_score {
        90..=100 => "Excellent",
        75..=89 => "Good",
        50..=74 => "Fair",
        _ => "Needs Attention",
    };
    let health_color_class = match health.health_score {
        90..=100 => "cc-health-excellent",
        75..=89 => "cc-health-good",
        50..=74 => "cc-health-fair",
        _ => "cc-health-attention",
    };
    let goal_rate = if health.total_sessions_30d > 0 {
        numeric::to_i64(
            numeric::to_f64(health.goals_achieved) / numeric::to_f64(health.total_sessions_30d)
                * 100.0,
        )
    } else {
        0
    };

    HealthData {
        has_health: health.total_sessions_30d > 0,
        health_obj: HealthObj {
            score: health.health_score,
            label: health_label,
            color_class: health_color_class,
            total_sessions_30d: health.total_sessions_30d,
            avg_quality: format!("{:.1}", health.avg_quality),
            goal_rate,
            top_suggestion: health.top_recommendation.clone(),
            has_suggestion: !health.top_recommendation.is_empty(),
        },
    }
}
