//! Data-collection orchestration for the site analytics dashboard.
//!
//! Runs only the queries the active tab renders; every `Result` collapses
//! into a logged default so a single failed query never takes the page down.
//! The code tab reads rollups the `usage_daily_rollup` job maintains, so it
//! renders zeros (not errors) until the first job run after deploy.

use std::sync::Arc;

use chrono::Datelike;
use sqlx::PgPool;

use crate::repositories::analytics::site::anomalies::UsageAnomalyRow;
use crate::repositories::analytics::site::code::{CodeDayBucket, CodeTotals};
use crate::repositories::analytics::site::kpis::{PermissionGrantStats, SiteKpis};
use crate::repositories::analytics::site::latency::LatencySplit;
use crate::repositories::analytics::site::leaderboards::{
    LeaderboardPage, LeaderboardSort, UserUsageRow,
};
use crate::repositories::analytics::site::model_series::ModelCostBucket;
use crate::repositories::analytics::site::seats::InactiveSeatRow;
use crate::repositories::analytics::site::series::{SeriesBucket, UsageBucket};
use crate::repositories::analytics::site::session_costs::SessionCostStats;
use crate::repositories::analytics::site::{
    SiteScope, anomalies, code, distribution, kpis, latency, leaderboards, model_series, seats,
    series, session_costs,
};
use crate::repositories::organizations::budget_warnings::{
    BudgetWarningHistoryRow, list_budget_warning_history,
};
use crate::repositories::organizations::metrics::{OrganizationMetrics, list_organization_metrics};
use crate::util::org_scope::OrgScope;
use crate::util::time_range::TimeRange;

use super::context::DashboardTab;

#[derive(Default)]
pub(super) struct AnalyticsDashboardData {
    pub kpis: SiteKpis,
    pub wasted_seat_count: i64,
    pub series: Vec<UsageBucket>,
    pub models: Vec<distribution::ModelDistributionRow>,
    pub leaderboard: Vec<UserUsageRow>,
    pub leaderboard_total: i64,
    pub permissions: PermissionGrantStats,
    pub inactive_seats: Vec<InactiveSeatRow>,
    pub org_metrics: Vec<OrganizationMetrics>,
    pub code_series: Vec<CodeDayBucket>,
    pub code_totals: CodeTotals,
    pub model_cost: Vec<ModelCostBucket>,
    pub session_costs: SessionCostStats,
    pub latency: LatencySplit,
    pub budget_history: Vec<BudgetWarningHistoryRow>,
    pub anomalies: Vec<UsageAnomalyRow>,
    pub mtd_series: Vec<UsageBucket>,
}

pub(super) struct DashboardQueryPlan<'a> {
    pub tab: DashboardTab,
    pub scope: &'a SiteScope,
    pub range: TimeRange,
    pub bucket: SeriesBucket,
    pub sort: LeaderboardSort,
    pub page_size: i64,
    pub offset: i64,
    pub all_orgs: bool,
    pub own_org_slug: Option<&'a str>,
    pub inactive_days: i32,
    pub slo_ms: i32,
}

pub(super) async fn load_dashboard_data(
    pool: &Arc<PgPool>,
    plan: DashboardQueryPlan<'_>,
) -> AnalyticsDashboardData {
    let mut data = AnalyticsDashboardData::default();

    // Why: loaded for every tab — the strip itself renders on Overview and
    // Usage, and the wasted-seat count is a KPI card link the others reuse.
    let (kpis_res, wasted_res) = tokio::join!(
        kpis::get_site_kpis(pool, plan.range, plan.scope),
        seats::count_inactive_seats(pool, plan.scope, plan.inactive_days),
    );
    data.kpis = kpis_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "get_site_kpis failed");
        SiteKpis::default()
    });
    data.wasted_seat_count = wasted_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "count_inactive_seats failed");
        0
    });

    match plan.tab {
        DashboardTab::Overview => {
            let (series_res, models_res, model_cost_res, orgs_res) = tokio::join!(
                series::list_daily_usage_series(pool, plan.range, plan.scope, plan.bucket),
                distribution::list_model_distribution(pool, plan.range, plan.scope),
                model_series::list_model_cost_series(pool, plan.range, plan.scope, plan.bucket),
                list_organization_metrics(pool),
            );
            data.series = unwrap_or_empty(series_res, "list_daily_usage_series");
            data.models = unwrap_or_empty(models_res, "list_model_distribution");
            data.model_cost = unwrap_or_empty(model_cost_res, "list_model_cost_series");
            data.org_metrics = scoped_orgs(orgs_res, &plan);
        },
        DashboardTab::Usage => load_usage_tab(pool, &plan, &mut data).await,
        DashboardTab::Seats => {
            let (seats_res, orgs_res) = tokio::join!(
                seats::list_inactive_seats(pool, plan.scope, plan.inactive_days),
                list_organization_metrics(pool),
            );
            data.inactive_seats = unwrap_or_empty(seats_res, "list_inactive_seats");
            data.org_metrics = scoped_orgs(orgs_res, &plan);
        },
        DashboardTab::Spend => load_spend_tab(pool, &plan, &mut data).await,
        DashboardTab::Code => {
            let (series_res, totals_res) = tokio::join!(
                code::list_daily_code_series(pool, plan.range, plan.scope),
                code::get_code_totals(pool, plan.range, plan.scope),
            );
            data.code_series = unwrap_or_empty(series_res, "list_daily_code_series");
            data.code_totals = totals_res.unwrap_or_else(|e| {
                tracing::warn!(error = %e, "get_code_totals failed");
                CodeTotals::default()
            });
        },
    }

    data
}

// Why: the spend tab is the one that pulls from three planes at once — org
// metrics, latency, and the budget-warning ledger — plus a second windowed
// series for the burn-up, so it lives in its own function rather than
// crowding the dispatch.
async fn load_spend_tab(
    pool: &PgPool,
    plan: &DashboardQueryPlan<'_>,
    data: &mut AnalyticsDashboardData,
) {
    let (orgs_res, latency_res, history_res, anomalies_res) = tokio::join!(
        list_organization_metrics(pool),
        latency::get_latency_split(pool, plan.range, plan.scope, plan.slo_ms),
        list_budget_warning_history(pool, &plan.scope.org_slug, 12),
        anomalies::list_recent_anomalies(pool, 10),
    );
    data.org_metrics = scoped_orgs(orgs_res, plan);
    data.latency = latency_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "get_latency_split failed");
        LatencySplit::default()
    });
    data.budget_history = history_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_budget_warning_history failed");
        Vec::new()
    });
    data.anomalies = unwrap_or_empty(anomalies_res, "list_recent_anomalies");
    // Why: a burn-up against a cap only means something for one
    // organization; the platform's all-orgs view gets a hint instead.
    if let OrgScope::Only(slug) = &plan.scope.org_slug {
        let month_scope = SiteScope {
            org_slug: OrgScope::Only(slug.clone()),
            department: None,
            user_id: None,
        };
        let mtd = series::list_daily_usage_series(
            pool,
            month_to_date_range(),
            &month_scope,
            SeriesBucket::Day,
        )
        .await;
        data.mtd_series = unwrap_or_empty(mtd, "list_daily_usage_series (mtd)");
    }
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

// Why: the burn-up always spans the calendar month the budget guard compares
// against, independent of the page's picked window.
fn month_to_date_range() -> TimeRange {
    let now = chrono::Utc::now();
    let month_start = now
        .date_naive()
        .with_day(1)
        .unwrap_or_else(|| now.date_naive())
        .and_hms_opt(0, 0, 0)
        .map_or(now, |d| d.and_utc());
    TimeRange {
        from: month_start,
        to: now,
        preset: crate::util::time_range::TimeRangePreset::Custom,
    }
}

// Why: org-level tables are visibility-scoped in code, not SQL — a platform
// admin sees every org (or the one the ?org filter picked), anyone else sees
// exactly their own.
fn scoped_orgs(
    res: Result<Vec<OrganizationMetrics>, systemprompt_web_shared::error::MarketplaceError>,
    plan: &DashboardQueryPlan<'_>,
) -> Vec<OrganizationMetrics> {
    let orgs = res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_organization_metrics failed");
        Vec::new()
    });
    orgs.into_iter()
        .filter(|o| {
            if plan.all_orgs {
                plan.scope
                    .org_slug
                    .as_slug()
                    .is_none_or(|slug| o.slug == slug)
            } else {
                plan.own_org_slug.is_some_and(|slug| o.slug == slug)
            }
        })
        .collect()
}

fn unwrap_or_empty<T>(res: Result<Vec<T>, sqlx::Error>, what: &'static str) -> Vec<T> {
    res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, query = what, "dashboard query failed");
        Vec::new()
    })
}
