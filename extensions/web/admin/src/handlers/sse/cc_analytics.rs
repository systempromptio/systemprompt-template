use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::{apm_metrics, hooks_track, session_analyses};
use crate::types::{
    conversation_analytics::{SessionEntityLink, SessionRating},
    ENTITY_SKILL,
};

use super::cc_types::{self, EntityLinkEntry};

pub(super) struct AnalyticsInput<'a> {
    pub entity_links: &'a [(String, SessionEntityLink)],
    pub session_ratings: &'a [SessionRating],
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

    let entity_data = build_entity_data(pool, user_id, &fetched.unused_skills).await;

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
        crate::repositories::conversation_analytics::fetch_skill_effectiveness(pool, user_id),
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
            tracing::error!(error = %e, "Failed to fetch unused skills");
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
}

async fn build_entity_data(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
    unused_skills: &[String],
) -> EntityData {
    let entity_usage =
        crate::repositories::conversation_analytics::fetch_entity_usage_summary(pool, user_id)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to fetch entity usage summary");
                Vec::new()
            });

    let skills_usage: Vec<_> = entity_usage
        .iter()
        .filter(|e| e.entity_type == ENTITY_SKILL)
        .collect();

    let total_skills_available = skills_usage.len() + unused_skills.len();
    let total_skills_used = skills_usage.len();
    let adoption_pct = (total_skills_used * 100)
        .checked_div(total_skills_available)
        .unwrap_or(0);

    EntityData {
        adoption_pct,
        total_skills_available,
        total_skills_used,
    }
}
