use crate::numeric;
use crate::repositories::session_analyses::HealthMetrics;
use crate::types::UserGamificationProfile;

use super::super::types::{HealthObj, SessionGroup};

pub struct SessionCounts {
    pub active: usize,
    pub analysed_closed: usize,
    pub has_any_data: bool,
}

pub fn build_session_counts(session_groups: &[SessionGroup]) -> SessionCounts {
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

pub struct GamificationData {
    pub has_gamification: bool,
    pub rank_level: i32,
    pub rank_name: String,
    pub total_xp: i64,
    pub xp_to_next: i64,
    pub next_rank_name: String,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub xp_progress_pct: i64,
}

pub fn build_gamification_data(
    gamification: Option<&UserGamificationProfile>,
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
        xp_progress_pct,
    ) = gamification.map_or((0, "", 0, 0, "", 0, 0, 0), |g| {
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
            pct,
        )
    });

    GamificationData {
        has_gamification,
        rank_level,
        rank_name: rank_name.to_string(),
        total_xp,
        xp_to_next,
        next_rank_name: next_rank_name.to_string(),
        current_streak,
        longest_streak,
        xp_progress_pct,
    }
}

pub struct HealthData {
    pub has_health: bool,
    pub health_obj: HealthObj,
}

pub fn build_health_data(health: &HealthMetrics) -> HealthData {
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
