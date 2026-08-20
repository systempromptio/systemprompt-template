//! Data-collection orchestration for the site analytics dashboard.
//!
//! Runs only the queries the active tab renders; every `Result` collapses
//! into a logged default so a single failed query never takes the page down.
//! The code tab reads rollups the `usage_daily_rollup` job maintains, so it
//! renders zeros (not errors) until the first job run after deploy.

use std::sync::Arc;

use sqlx::PgPool;

use crate::repositories::analytics::site::code::{CodeDayBucket, CodeTotals};
use crate::repositories::analytics::site::kpis::{PermissionGrantStats, SiteKpis};
use crate::repositories::analytics::site::leaderboards::{
    LeaderboardPage, LeaderboardSort, UserUsageRow,
};
use crate::repositories::analytics::site::seats::InactiveSeatRow;
use crate::repositories::analytics::site::series::{SeriesBucket, UsageBucket};
use crate::repositories::analytics::site::{
    SiteScope, code, distribution, kpis, leaderboards, seats, series,
};
use crate::repositories::organizations::metrics::{OrganizationMetrics, list_organization_metrics};
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
}

pub(super) struct DashboardQueryPlan<'a> {
    pub tab: DashboardTab,
    pub scope: &'a SiteScope,
    pub range: TimeRange,
    pub bucket: SeriesBucket,
    pub sort: LeaderboardSort,
    pub page_size: i64,
    pub offset: i64,
    /// Whether the viewer may see every organization's rows (platform admin).
    pub all_orgs: bool,
    /// The viewer's own org slug, for scoping org-level tables when they are
    /// not a platform admin.
    pub own_org_slug: Option<&'a str>,
}

pub(super) async fn load_dashboard_data(
    pool: &Arc<PgPool>,
    plan: DashboardQueryPlan<'_>,
) -> AnalyticsDashboardData {
    let mut data = AnalyticsDashboardData::default();

    // Why: the KPI strip and the wasted-seat count render on every tab.
    let (kpis_res, wasted_res) = tokio::join!(
        kpis::get_site_kpis(pool, plan.range, plan.scope),
        seats::count_inactive_seats(pool, plan.scope),
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
            let (series_res, models_res, orgs_res) = tokio::join!(
                series::list_daily_usage_series(pool, plan.range, plan.scope, plan.bucket),
                distribution::list_model_distribution(pool, plan.range, plan.scope),
                list_organization_metrics(pool),
            );
            data.series = unwrap_or_empty(series_res, "list_daily_usage_series");
            data.models = unwrap_or_empty(models_res, "list_model_distribution");
            data.org_metrics = scoped_orgs(orgs_res, &plan);
        },
        DashboardTab::Usage => load_usage_tab(pool, &plan, &mut data).await,
        DashboardTab::Seats => {
            let (seats_res, orgs_res) = tokio::join!(
                seats::list_inactive_seats(pool, plan.scope),
                list_organization_metrics(pool),
            );
            data.inactive_seats = unwrap_or_empty(seats_res, "list_inactive_seats");
            data.org_metrics = scoped_orgs(orgs_res, &plan);
        },
        DashboardTab::Spend => {
            data.org_metrics = scoped_orgs(list_organization_metrics(pool).await, &plan);
        },
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

async fn load_usage_tab(
    pool: &PgPool,
    plan: &DashboardQueryPlan<'_>,
    data: &mut AnalyticsDashboardData,
) {
    let (leaders_res, perms_res) = tokio::join!(
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
                    .as_deref()
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
