//! Data-collection orchestration for the site analytics dashboard.
//!
//! Runs only the queries the active tab renders; every `Result` collapses
//! into a logged default so a single failed query never takes the page down.
//! The Overview tab is the exception and loads every site aggregate at once,
//! because its job is to put the whole instance on one screen — the five
//! entity tabs then each pay for their own queries alone.
//!
//! The code figures read rollups the `usage_daily_rollup` job maintains, so
//! they render zeros (not errors) until the first job run after deploy.

use std::sync::Arc;

use sqlx::PgPool;

use crate::repositories::analytics::site::anomalies::UsageAnomalyRow;
use crate::repositories::analytics::site::code::{CodeDayBucket, CodeTotals};
use crate::repositories::analytics::site::kpis::{PermissionGrantStats, SiteKpis};
use crate::repositories::analytics::site::latency::LatencySplit;
use crate::repositories::analytics::site::leaderboards::{
    LeaderboardPage, LeaderboardSort, UserUsageRow,
};
use crate::repositories::analytics::site::model_series::ModelCostBucket;
use crate::repositories::analytics::site::series::{SeriesBucket, UsageBucket};
use crate::repositories::analytics::site::session_costs::SessionCostStats;
use crate::repositories::analytics::site::{
    SiteScope, anomalies, code, distribution, kpis, latency, leaderboards, model_series, series,
    session_costs,
};
use crate::util::time_range::TimeRange;

use super::context::DashboardTab;
use super::data_tabs::{self, TabData, TabPlan};

#[derive(Default)]
pub(super) struct AnalyticsDashboardData {
    pub kpis: SiteKpis,
    pub series: Vec<UsageBucket>,
    pub models: Vec<distribution::ModelDistributionRow>,
    pub leaderboard: Vec<UserUsageRow>,
    pub leaderboard_total: i64,
    pub permissions: PermissionGrantStats,
    pub code_series: Vec<CodeDayBucket>,
    pub code_totals: CodeTotals,
    pub model_cost: Vec<ModelCostBucket>,
    pub session_costs: SessionCostStats,
    pub latency: LatencySplit,
    pub anomalies: Vec<UsageAnomalyRow>,
    pub tabs: TabData,
}

pub(super) struct DashboardQueryPlan<'a> {
    pub tab: DashboardTab,
    pub scope: &'a SiteScope,
    pub range: TimeRange,
    pub bucket: SeriesBucket,
    pub sort: LeaderboardSort,
    pub page_size: i64,
    pub offset: i64,
    pub slo_ms: i32,
    pub axis: crate::repositories::analytics::site::cost::ContainerAxis,
}

pub(super) async fn load_dashboard_data(
    pool: &Arc<PgPool>,
    plan: DashboardQueryPlan<'_>,
) -> AnalyticsDashboardData {
    // Why: loaded for every tab — the strip renders on Overview and Usage,
    // and the other tabs share its deltas.
    let kpis = kpis::get_site_kpis(pool, plan.range, plan.scope)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "get_site_kpis failed");
            SiteKpis::default()
        });
    let mut data = AnalyticsDashboardData {
        kpis,
        ..AnalyticsDashboardData::default()
    };

    let tab_plan = TabPlan {
        scope: plan.scope,
        range: plan.range,
        limit: plan.page_size,
        offset: plan.offset,
        axis: plan.axis,
    };
    match plan.tab {
        DashboardTab::Overview => load_overview(pool, &plan, &mut data).await,
        DashboardTab::Models => data.tabs = data_tabs::load_models(pool, &tab_plan).await,
        DashboardTab::Skills => data.tabs = data_tabs::load_skills(pool, &tab_plan).await,
        DashboardTab::Tools => data.tabs = data_tabs::load_tools(pool, &tab_plan).await,
        DashboardTab::Sessions => data.tabs = data_tabs::load_sessions(pool, &tab_plan).await,
        DashboardTab::Cost => data.tabs = data_tabs::load_cost(pool, &tab_plan).await,
    }

    data
}

// Why: the Overview is the one screen that must answer "how is the instance
// doing" without a click, so it pays for every aggregate at once. The tabs
// below it exist precisely so that cost is not paid twice.
async fn load_overview(
    pool: &PgPool,
    plan: &DashboardQueryPlan<'_>,
    data: &mut AnalyticsDashboardData,
) {
    let (series_res, models_res, model_cost_res) = tokio::join!(
        series::list_daily_usage_series(pool, plan.range, plan.scope, plan.bucket),
        distribution::list_model_distribution(pool, plan.range, plan.scope),
        model_series::list_model_cost_series(pool, plan.range, plan.scope, plan.bucket),
    );
    data.series = unwrap_or_empty(series_res, "list_daily_usage_series");
    data.models = unwrap_or_empty(models_res, "list_model_distribution");
    data.model_cost = unwrap_or_empty(model_cost_res, "list_model_cost_series");

    load_usage_tab(pool, plan, data).await;
    load_spend_tab(pool, plan, data).await;

    let (code_series_res, code_totals_res) = tokio::join!(
        code::list_daily_code_series(pool, plan.range, plan.scope),
        code::get_code_totals(pool, plan.range, plan.scope),
    );
    data.code_series = unwrap_or_empty(code_series_res, "list_daily_code_series");
    data.code_totals = code_totals_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "get_code_totals failed");
        CodeTotals::default()
    });
}

async fn load_spend_tab(
    pool: &PgPool,
    plan: &DashboardQueryPlan<'_>,
    data: &mut AnalyticsDashboardData,
) {
    let (latency_res, anomalies_res) = tokio::join!(
        latency::get_latency_split(pool, plan.range, plan.scope, plan.slo_ms),
        anomalies::list_recent_anomalies(pool, plan.range, 10),
    );
    data.latency = latency_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "get_latency_split failed");
        LatencySplit::default()
    });
    data.anomalies = unwrap_or_empty(anomalies_res, "list_recent_anomalies");
}

async fn load_usage_tab(
    pool: &PgPool,
    plan: &DashboardQueryPlan<'_>,
    data: &mut AnalyticsDashboardData,
) {
    let (leaders_res, perms_res, session_costs_res) = tokio::join!(
        leaderboards::list_top_users_by_requests(
            pool,
            plan.range,
            plan.scope,
            LeaderboardPage {
                sort: plan.sort,
                limit: plan.page_size,
                offset: plan.offset,
            },
        ),
        kpis::get_permission_grant_stats(pool, plan.range, plan.scope),
        session_costs::get_session_cost_stats(pool, plan.range, plan.scope),
    );
    match leaders_res {
        Ok((rows, total)) => {
            data.leaderboard = rows;
            data.leaderboard_total = total;
        },
        Err(e) => tracing::warn!(error = %e, "list_top_users_by_requests failed"),
    }
    data.permissions = perms_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "get_permission_grant_stats failed");
        PermissionGrantStats::default()
    });
    data.session_costs = session_costs_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "get_session_cost_stats failed");
        SessionCostStats::default()
    });
}

pub(super) fn unwrap_or_empty<T>(res: Result<Vec<T>, sqlx::Error>, what: &'static str) -> Vec<T> {
    res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, query = what, "dashboard query failed");
        Vec::new()
    })
}
