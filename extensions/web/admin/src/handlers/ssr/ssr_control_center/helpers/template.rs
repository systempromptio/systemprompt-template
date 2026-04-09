use serde::Serialize;

use super::apm;
use super::data;
use super::data_analyses;
use super::BuildTemplateDataParams;
use crate::handlers::ssr::ssr_control_center::types::{
    SessionGroup, SkillEffectivenessEntry, StarRating,
};

pub(crate) struct AssembleInput<'a> {
    pub params: &'a BuildTemplateDataParams<'a>,
    pub counts: data::SessionCounts,
    pub gam: data::GamificationData,
    pub health: data::HealthData,
    pub analyses: data_analyses::AnalysesData,
    pub adoption: data_analyses::SkillAdoption,
    pub apm: apm::ApmData,
    pub report: super::super::types::ReportData,
}

#[derive(Serialize)]
pub(crate) struct SectionFlags {
    pub has_skill_adoption: bool,
    pub has_today_summary: bool,
    pub has_report: bool,
}

#[derive(Serialize)]
pub(crate) struct TemplateData<'a> {
    pub page: &'static str,
    pub title: &'static str,
    pub cc_initial_json: &'a str,
    pub username: &'a str,
    pub tier_name: &'a str,
    pub has_any_data: bool,
    pub status_filter: &'a str,
    pub active_count: usize,
    pub analysed_count: usize,
    pub unanalysed_count: usize,
    pub total_session_count: usize,
    pub has_gamification: bool,
    pub rank_level: i32,
    pub rank_name: &'a str,
    pub total_xp: i64,
    pub xp_to_next_rank: i64,
    pub next_rank_name: &'a str,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub achievements_count: usize,
    pub achievements_total: usize,
    pub xp_progress_pct: i64,
    pub today: &'a super::super::types::TodayObj,
    pub session_groups: &'a [SessionGroup],
    pub top_tools: &'a [serde_json::Value],
    pub skill_effectiveness: Vec<SkillEffectivenessEntry>,
    pub skills_usage: &'a [&'a crate::types::conversation_analytics::EntityUsageSummary],
    pub agents_usage: &'a [&'a crate::types::conversation_analytics::EntityUsageSummary],
    pub mcp_usage: &'a [&'a crate::types::conversation_analytics::EntityUsageSummary],
    pub skill_ratings: &'a [crate::types::conversation_analytics::SkillRating],
    pub has_health: bool,
    pub health: &'a super::super::types::HealthObj,
    pub recent_analyses: &'a [super::super::types::AnalysisEntry],
    pub recommendations: &'a [super::super::types::AnalysisEntry],
    pub unused_skills: &'a [String],
    pub skill_adoption_pct: usize,
    pub total_skills_available: usize,
    pub total_skills_used: usize,
    pub achievement_progress: &'a [super::super::types::AchievementProgress],
    pub today_summary: &'a super::super::types::TodaySummaryObj,
    pub hourly_breakdown: &'a [super::super::types::HourlyEntry],
    pub performance_summary: &'a super::super::types::PerfSummaryObj,
    pub report: &'a super::super::types::ReportData,
    #[serde(flatten)]
    pub section_flags: SectionFlags,
}

pub(crate) fn assemble_template<'a>(input: &'a AssembleInput<'a>) -> TemplateData<'a> {
    let AssembleInput {
        params,
        counts,
        gam,
        health,
        analyses,
        adoption,
        apm,
        report,
    } = input;

    let skill_effectiveness: Vec<SkillEffectivenessEntry> = params
        .skill_effectiveness
        .iter()
        .map(|s| SkillEffectivenessEntry {
            skill_name: s.skill_name.clone(),
            total_uses: s.total_uses,
            sessions_used_in: s.sessions_used_in,
            avg_effectiveness: format!("{:.1}", s.avg_effectiveness),
            scored_sessions: s.scored_sessions,
            goal_achievement_pct: format!("{:.0}", s.goal_achievement_pct),
            has_score: s.scored_sessions > 0,
            stars: StarRating {
                rating: u8::try_from(s.avg_effectiveness.round() as i64).unwrap_or(0),
            },
        })
        .collect();

    TemplateData {
        page: "control-center",
        title: "Control Center",
        cc_initial_json: &apm.cc_initial_json,
        username: &params.user_ctx.username,
        tier_name: &params.mkt_ctx.tier_name,
        has_any_data: counts.has_any_data,
        status_filter: params.status_filter,
        active_count: counts.active,
        analysed_count: counts.analysed_closed,
        unanalysed_count: params.session_groups.len() - counts.analysed_closed,
        total_session_count: params.session_groups.len(),
        has_gamification: gam.has_gamification,
        rank_level: gam.rank_level,
        rank_name: &gam.rank_name,
        total_xp: gam.total_xp,
        xp_to_next_rank: gam.xp_to_next,
        next_rank_name: &gam.next_rank_name,
        current_streak: gam.current_streak,
        longest_streak: gam.longest_streak,
        achievements_count: gam.achievements_count,
        achievements_total: gam.achievements_total,
        xp_progress_pct: gam.xp_progress_pct,
        today: &apm.today_obj,
        session_groups: params.session_groups,
        top_tools: params.tools_with_pct,
        skill_effectiveness,
        skills_usage: params.skills_usage,
        agents_usage: params.agents_usage,
        mcp_usage: params.mcp_usage,
        skill_ratings: params.skill_ratings,
        has_health: health.has_health,
        health: &health.health_obj,
        recent_analyses: &analyses.recent_analyses_json,
        recommendations: &analyses.recommendations_json,
        unused_skills: params.unused_skills,
        skill_adoption_pct: adoption.adoption_pct,
        total_skills_available: adoption.total_available,
        total_skills_used: adoption.total_used,
        achievement_progress: &gam.achievement_progress,
        today_summary: &apm.today_summary_obj,
        hourly_breakdown: &apm.hourly_json,
        performance_summary: &apm.perf_json,
        report,
        section_flags: SectionFlags {
            has_skill_adoption: adoption.total_available > 0,
            has_today_summary: params.today_summary.sessions_count > 0,
            has_report: !params.daily_summaries.is_empty(),
        },
    }
}
