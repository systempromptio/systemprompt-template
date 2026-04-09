mod apm;
mod data;
mod data_analyses;
mod groups;
pub(in crate::admin) mod metrics;
pub(in crate::admin) mod report;
pub(in crate::admin) mod report_sections;
mod template;

use sqlx::PgPool;

use crate::admin::gamification;
use crate::admin::repositories;
use crate::admin::repositories::apm_metrics::{
    HourlyApmBucket, TodayApmLive, TodayPerformanceSummary,
};
use crate::admin::repositories::control_center;
use crate::admin::repositories::daily_summaries::DailySummaryRow;
use crate::admin::repositories::session_analyses::{
    HealthMetrics, SessionAnalysisRow, TodaySummary,
};
use crate::admin::types::{MarketplaceContext, UserContext, UserGamificationProfile};

use apm::build_apm_data;
use data::{build_gamification_data, build_health_data, build_session_counts};
use data_analyses::{build_analyses_data, build_skill_adoption};
pub(super) use groups::{build_session_groups, partition_entity_usage};
use report::{build_report_data, ReportParams};
use template::{assemble_template, AssembleInput};

use super::types::{EntityCounts, SessionGroup};

pub(super) struct AnalyticsData {
    pub skill_effectiveness: Vec<crate::admin::types::conversation_analytics::SkillEffectiveness>,
    pub entity_usage: Vec<crate::admin::types::conversation_analytics::EntityUsageSummary>,
    pub session_ratings: Vec<crate::admin::types::conversation_analytics::SessionRating>,
    pub skill_ratings: Vec<crate::admin::types::conversation_analytics::SkillRating>,
    pub entity_links: Vec<(
        String,
        crate::admin::types::conversation_analytics::SessionEntityLink,
    )>,
    pub unused_skills: Vec<String>,
    pub today_summary: TodaySummary,
}

pub(super) async fn fetch_control_center_data(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
    status_filter: &str,
) -> (
    Vec<crate::admin::types::control_center::RecentSession>,
    crate::admin::types::control_center::TodayStats,
    control_center::TodayOutcomeStats,
    Vec<crate::admin::types::ToolUsageCount>,
    AnalyticsData,
    Option<UserGamificationProfile>,
    TodayApmLive,
    Vec<HourlyApmBucket>,
    TodayPerformanceSummary,
) {
    let (
        recent_sessions_res,
        today_stats,
        outcome_stats,
        top_tools_res,
        skill_effectiveness_res,
        entity_usage_res,
        session_ratings_res,
        skill_ratings_res,
        entity_links_res,
        gamification_res,
        unused_skills_res,
        today_summary,
        apm_live,
        hourly_breakdown,
        perf_summary,
    ) = tokio::join!(
        control_center::fetch_recent_sessions_filtered(pool, user_id, 50, status_filter),
        control_center::fetch_today_stats(pool, user_id),
        control_center::fetch_today_outcome_stats(pool, user_id),
        repositories::get_user_top_tools(pool, user_id),
        repositories::conversation_analytics::fetch_skill_effectiveness(pool, user_id),
        repositories::conversation_analytics::fetch_entity_usage_summary(pool, user_id),
        repositories::conversation_analytics::fetch_all_session_ratings(pool, user_id),
        repositories::conversation_analytics::fetch_all_skill_ratings(pool, user_id),
        repositories::conversation_analytics::fetch_all_session_entity_links(pool, user_id),
        gamification::queries::find_user_gamification(pool, user_id.as_str()),
        repositories::conversation_analytics::fetch_unused_skills(pool, user_id),
        repositories::session_analyses::fetch_today_summary(pool, user_id),
        repositories::apm_metrics::fetch_today_apm_live(pool, user_id.as_str()),
        repositories::apm_metrics::fetch_hourly_breakdown(pool, user_id.as_str()),
        repositories::apm_metrics::fetch_today_performance_summary(pool, user_id.as_str()),
    );
    (
        recent_sessions_res.unwrap_or_else(|_| Vec::new()),
        today_stats,
        outcome_stats,
        top_tools_res.unwrap_or_else(|_| Vec::new()),
        AnalyticsData {
            skill_effectiveness: skill_effectiveness_res.unwrap_or_else(|_| Vec::new()),
            entity_usage: entity_usage_res.unwrap_or_else(|_| Vec::new()),
            session_ratings: session_ratings_res.unwrap_or_else(|_| Vec::new()),
            skill_ratings: skill_ratings_res.unwrap_or_else(|_| Vec::new()),
            entity_links: entity_links_res.unwrap_or_else(|_| Vec::new()),
            unused_skills: unused_skills_res.unwrap_or_else(|_| Vec::new()),
            today_summary,
        },
        gamification_res.unwrap_or(None),
        apm_live,
        hourly_breakdown,
        perf_summary,
    )
}

pub(super) struct BuildTemplateDataParams<'a> {
    pub user_ctx: &'a UserContext,
    pub mkt_ctx: &'a MarketplaceContext,
    pub status_filter: &'a str,
    pub today_stats: &'a crate::admin::types::control_center::TodayStats,
    pub outcome_stats: &'a control_center::TodayOutcomeStats,
    pub session_groups: &'a [SessionGroup],
    pub tools_with_pct: &'a [serde_json::Value],
    pub skill_effectiveness:
        &'a [crate::admin::types::conversation_analytics::SkillEffectiveness],
    pub skills_usage: &'a [&'a crate::admin::types::conversation_analytics::EntityUsageSummary],
    pub agents_usage: &'a [&'a crate::admin::types::conversation_analytics::EntityUsageSummary],
    pub mcp_usage: &'a [&'a crate::admin::types::conversation_analytics::EntityUsageSummary],
    pub skill_ratings: &'a [crate::admin::types::conversation_analytics::SkillRating],
    pub gamification: Option<&'a UserGamificationProfile>,
    pub health_metrics: &'a HealthMetrics,
    pub recent_analyses: &'a [SessionAnalysisRow],
    pub unused_skills: &'a [String],
    pub today_summary: &'a TodaySummary,
    pub apm_live: &'a TodayApmLive,
    pub hourly_breakdown: &'a [HourlyApmBucket],
    pub perf_summary: &'a TodayPerformanceSummary,
    pub daily_summaries: &'a [DailySummaryRow],
    pub global_averages: &'a repositories::daily_summaries::GlobalAverages,
    pub entity_counts: &'a EntityCounts,
    pub current_streak: i32,
}

pub(super) fn build_template_data(params: &BuildTemplateDataParams<'_>) -> serde_json::Value {
    let counts = build_session_counts(params.session_groups);
    let gamification_data =
        build_gamification_data(params.gamification, params.skills_usage, params.mcp_usage);
    let health_data = build_health_data(params.health_metrics);
    let analyses_data = build_analyses_data(params.recent_analyses);
    let adoption = build_skill_adoption(params.skills_usage, params.unused_skills);
    let apm_data = build_apm_data(params);

    let report = build_report_data(&ReportParams {
        perf: params.perf_summary,
        today: params.today_summary,
        health: params.health_metrics,
        daily_summaries: params.daily_summaries,
        global: params.global_averages,
        entity_counts: params.entity_counts,
        current_streak: params.current_streak,
        recent_analyses: params.recent_analyses,
    });

    let input = AssembleInput {
        params,
        counts,
        gam: gamification_data,
        health: health_data,
        analyses: analyses_data,
        adoption,
        apm: apm_data,
        report,
    };
    let template = assemble_template(&input);

    serde_json::to_value(&template).unwrap_or(serde_json::Value::Null)
}
