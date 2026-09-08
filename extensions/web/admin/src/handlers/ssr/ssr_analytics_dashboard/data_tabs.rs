//! Data collection for the five entity tabs.
//!
//! Each tab loads only its own queries, so opening Models never pays for the
//! skill view. Every `Result` collapses into a logged empty default: one
//! failing aggregate must degrade its own table, not take the page down.

use sqlx::PgPool;

use crate::repositories::analytics::site::cost::{
    ContainerAxis, ContainerUsageRow, CostDayRow, SupplierCostRow, list_container_usage,
    list_provider_cost_by_day, list_provider_costs,
};
use crate::repositories::analytics::site::models::{
    ModelRedirectRow, ModelStatsRow, list_model_redirects, list_model_stats,
};
use crate::repositories::analytics::site::sessions::{
    SessionCostRow, SessionRatingStats, get_session_rating_stats, list_session_costs_paged,
};
use crate::repositories::analytics::site::skills::{
    SkillModelRow, SkillStatsRow, SkillTotals, get_skill_totals, list_skill_by_model,
    list_skill_stats,
};
use crate::repositories::analytics::site::tools::{
    ToolServerRow, ToolStatsRow, list_tool_servers, list_tool_stats,
};
use crate::repositories::analytics::site::{SiteScope, distribution};
use crate::util::time_range::TimeRange;

use super::data::unwrap_or_empty;

#[derive(Default)]
pub(super) struct TabData {
    pub models: Vec<ModelStatsRow>,
    pub redirects: Vec<ModelRedirectRow>,
    pub skills: Vec<SkillStatsRow>,
    pub skills_total: i64,
    pub skill_models: Vec<SkillModelRow>,
    pub skill_totals: SkillTotals,
    pub tool_servers: Vec<ToolServerRow>,
    pub tools: Vec<ToolStatsRow>,
    pub tools_total: i64,
    pub sessions: Vec<SessionCostRow>,
    pub sessions_total: i64,
    pub session_ratings: SessionRatingStats,
    pub cost_days: Vec<CostDayRow>,
    pub cost_providers: Vec<SupplierCostRow>,
    pub cost_models: Vec<SupplierCostRow>,
    pub cost_containers: Vec<ContainerUsageRow>,
}

pub(super) struct TabPlan<'a> {
    pub scope: &'a SiteScope,
    pub range: TimeRange,
    pub limit: i64,
    pub offset: i64,
    pub axis: ContainerAxis,
}

pub(super) async fn load_models(pool: &PgPool, plan: &TabPlan<'_>) -> TabData {
    let (stats, redirects) = tokio::join!(
        list_model_stats(pool, plan.range, plan.scope),
        list_model_redirects(pool, plan.range, plan.scope),
    );
    TabData {
        models: unwrap_or_empty(stats, "list_model_stats"),
        redirects: unwrap_or_empty(redirects, "list_model_redirects"),
        ..TabData::default()
    }
}

pub(super) async fn load_skills(pool: &PgPool, plan: &TabPlan<'_>) -> TabData {
    let (paged, by_model, totals) = tokio::join!(
        list_skill_stats(pool, plan.range, plan.scope, plan.limit, plan.offset),
        list_skill_by_model(pool, plan.range, plan.scope),
        get_skill_totals(pool, plan.range, plan.scope),
    );
    let (skills, skills_total) = unwrap_paged(paged, "list_skill_stats");
    TabData {
        skills,
        skills_total,
        skill_models: unwrap_or_empty(by_model, "list_skill_by_model"),
        skill_totals: totals.unwrap_or_else(|e| {
            tracing::warn!(error = %e, "get_skill_totals failed");
            SkillTotals::default()
        }),
        ..TabData::default()
    }
}

pub(super) async fn load_tools(pool: &PgPool, plan: &TabPlan<'_>) -> TabData {
    let (servers, paged) = tokio::join!(
        list_tool_servers(pool, plan.range, plan.scope),
        list_tool_stats(pool, plan.range, plan.scope, plan.limit, plan.offset),
    );
    let (tools, tools_total) = unwrap_paged(paged, "list_tool_stats");
    TabData {
        tool_servers: unwrap_or_empty(servers, "list_tool_servers"),
        tools,
        tools_total,
        ..TabData::default()
    }
}

pub(super) async fn load_sessions(pool: &PgPool, plan: &TabPlan<'_>) -> TabData {
    let (paged, ratings) = tokio::join!(
        list_session_costs_paged(pool, plan.range, plan.scope, plan.limit, plan.offset),
        get_session_rating_stats(pool, plan.range, plan.scope),
    );
    let (sessions, sessions_total) = unwrap_paged(paged, "list_session_costs_paged");
    TabData {
        sessions,
        sessions_total,
        session_ratings: ratings.unwrap_or_else(|e| {
            tracing::warn!(error = %e, "get_session_rating_stats failed");
            SessionRatingStats::default()
        }),
        ..TabData::default()
    }
}

pub(super) async fn load_cost(pool: &PgPool, plan: &TabPlan<'_>) -> TabData {
    let (days, providers, models, containers) = tokio::join!(
        list_provider_cost_by_day(pool, plan.range, plan.scope),
        list_provider_costs(pool, plan.range, plan.scope),
        distribution::list_model_distribution(pool, plan.range, plan.scope),
        list_container_usage(pool, plan.range, plan.scope, plan.axis),
    );
    TabData {
        cost_days: unwrap_or_empty(days, "list_provider_cost_by_day"),
        cost_providers: unwrap_or_empty(providers, "list_provider_costs"),
        // Why: the model split is the same distribution the Overview pie
        // reads, re-labelled as a supplier line rather than queried twice.
        cost_models: unwrap_or_empty(models, "list_model_distribution")
            .into_iter()
            .map(|m| SupplierCostRow {
                label: m.model,
                requests: m.requests,
                tokens: m.tokens,
                cost_microdollars: m.cost_microdollars,
            })
            .collect(),
        cost_containers: unwrap_or_empty(containers, "list_container_usage"),
        ..TabData::default()
    }
}

fn unwrap_paged<T>(res: Result<(Vec<T>, i64), sqlx::Error>, what: &'static str) -> (Vec<T>, i64) {
    res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, query = what, "paged dashboard query failed");
        (Vec::new(), 0)
    })
}
