use std::collections::HashSet;

use serde::Serialize;
use sqlx::PgPool;

use crate::numeric;
use crate::repositories::{apm_metrics, hooks_track, session_analyses};
use crate::types::{
    conversation_analytics::{SessionEntityLink, SessionRating},
    UserGamificationProfile,
};

use super::cc_types::{self, AchievementProgressEntry, EntityLinkEntry};

pub(super) struct AnalyticsInput<'a> {
    pub entity_links: &'a [(String, SessionEntityLink)],
    pub session_ratings: &'a [SessionRating],
    pub gam: Option<&'a UserGamificationProfile>,
    pub apm_live: &'a apm_metrics::TodayApmLive,
}

#[derive(Serialize)]
struct AnalyticsEvent<'a> {
    skill_effectiveness: Vec<crate::types::conversation_analytics::SkillEffectiveness>,
    entity_links: Vec<EntityLinkEntry<'a>>,
    session_ratings: &'a [SessionRating],
    health: Option<cc_types::HealthEntry>,
    unused_skills: Vec<String>,
    skill_adoption_pct: usize,
    total_skills_available: usize,
    total_skills_used: usize,
    today_summary: cc_types::TodaySummaryEntry,
    achievement_progress: Vec<AchievementProgressEntry>,
    hourly_breakdown: Vec<cc_types::HourlyEntry>,
    performance_summary: cc_types::PerformanceEntry,
}

pub(super) async fn build_analytics_event(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
    input: &AnalyticsInput<'_>,
) -> Option<String> {
    let fetched = fetch_analytics_data(pool, user_id).await;
    let skills = fetched.skill_eff_res.ok()?;

    let health = cc_types::build_health(&fetched.health_metrics);
    let today_summary = cc_types::build_today_summary(
        &fetched.today_summary,
        input.apm_live,
        &fetched.apm_correlation,
        &fetched.achievements_today,
    );
    let hourly_breakdown = cc_types::build_hourly(&fetched.hourly_breakdown);
    let performance_summary = cc_types::build_performance(&fetched.perf_summary);

    let entity_data = build_entity_data(pool, user_id, &fetched.unused_skills, input.gam).await;

    let entity_links: Vec<EntityLinkEntry<'_>> = input
        .entity_links
        .iter()
        .map(|(sid, link)| EntityLinkEntry {
            session_id: sid,
            entity_type: &link.entity_type,
            entity_name: &link.entity_name,
            usage_count: link.usage_count,
        })
        .collect();

    let event = AnalyticsEvent {
        skill_effectiveness: skills,
        entity_links,
        session_ratings: input.session_ratings,
        health,
        unused_skills: fetched.unused_skills,
        skill_adoption_pct: entity_data.adoption_pct,
        total_skills_available: entity_data.total_skills_available,
        total_skills_used: entity_data.total_skills_used,
        today_summary,
        achievement_progress: entity_data.achievement_progress,
        hourly_breakdown,
        performance_summary,
    };

    serde_json::to_string(&event).ok()
}

struct FetchedAnalytics {
    skill_eff_res:
        Result<Vec<crate::types::conversation_analytics::SkillEffectiveness>, sqlx::Error>,
    health_metrics: session_analyses::HealthMetrics,
    unused_skills: Vec<String>,
    today_summary: session_analyses::TodaySummary,
    apm_correlation: apm_metrics::ApmCorrelation,
    achievements_today: Vec<String>,
    hourly_breakdown: Vec<apm_metrics::HourlyApmBucket>,
    perf_summary: apm_metrics::TodayPerformanceSummary,
}

async fn fetch_analytics_data(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
) -> FetchedAnalytics {
    let (
        skill_eff_res,
        health_metrics,
        unused_skills_res,
        today_summary,
        apm_correlation,
        achievements_today,
        hourly_breakdown,
        perf_summary,
    ) = tokio::join!(
        crate::repositories::conversation_analytics::fetch_skill_effectiveness(
            pool, user_id
        ),
        session_analyses::fetch_health_metrics(pool, user_id),
        crate::repositories::conversation_analytics::fetch_unused_skills(pool, user_id),
        session_analyses::fetch_today_summary(pool, user_id),
        apm_metrics::fetch_apm_success_correlation(pool, user_id.as_str()),
        hooks_track::fetch_today_achievements(pool, user_id.as_str()),
        apm_metrics::fetch_hourly_breakdown(pool, user_id.as_str()),
        apm_metrics::fetch_today_performance_summary(pool, user_id.as_str()),
    );

    FetchedAnalytics {
        skill_eff_res,
        health_metrics,
        unused_skills: unused_skills_res.unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch unused skills");
            Vec::new()
        }),
        today_summary,
        apm_correlation,
        achievements_today,
        hourly_breakdown,
        perf_summary,
    }
}

struct EntityData {
    adoption_pct: usize,
    total_skills_available: usize,
    total_skills_used: usize,
    achievement_progress: Vec<AchievementProgressEntry>,
}

async fn build_entity_data(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
    unused_skills: &[String],
    gam: Option<&UserGamificationProfile>,
) -> EntityData {
    let entity_usage =
        crate::repositories::conversation_analytics::fetch_entity_usage_summary(
            pool, user_id,
        )
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch entity usage summary");
            Vec::new()
        });

    let skills_usage: Vec<_> = entity_usage
        .iter()
        .filter(|e| e.entity_type == "skill")
        .collect();
    let mcp_usage: Vec<_> = entity_usage
        .iter()
        .filter(|e| e.entity_type == "mcp_tool")
        .collect();

    let total_skills_available = skills_usage.len() + unused_skills.len();
    let total_skills_used = skills_usage.len();
    let adoption_pct = if total_skills_available > 0 {
        (total_skills_used * 100) / total_skills_available
    } else {
        0
    };

    let achievement_progress = build_achievement_progress(gam, &skills_usage, &mcp_usage);

    EntityData {
        adoption_pct,
        total_skills_available,
        total_skills_used,
        achievement_progress,
    }
}

fn build_achievement_progress(
    gam: Option<&UserGamificationProfile>,
    skills_usage: &[&crate::types::conversation_analytics::EntityUsageSummary],
    mcp_usage: &[&crate::types::conversation_analytics::EntityUsageSummary],
) -> Vec<AchievementProgressEntry> {
    let mut result = Vec::new();
    let Some(g) = gam else {
        return result;
    };

    let unlocked_ids: HashSet<&str> = g
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
            result.push(AchievementProgressEntry {
                id,
                name,
                current,
                threshold,
                remaining,
                pct,
            });
        }
    }
    result
}
