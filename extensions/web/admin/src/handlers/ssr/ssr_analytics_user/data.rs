//! Data collection for the per-user analytics page.
//!
//! One `tokio::join!` over the whole page; every `Result` collapses into a
//! logged default so one failed query renders an empty panel rather than
//! taking the page down — the same rule the site dashboard follows.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::analytics::site::code::{CodeDayBucket, CodeTotals};
use crate::repositories::analytics::site::distribution::ModelDistributionRow;
use crate::repositories::analytics::site::kpis::{PermissionGrantStats, SiteKpis};
use crate::repositories::analytics::site::series::{SeriesBucket, UsageBucket};
use crate::repositories::analytics::site::session_costs::UserSessionCostRow;
use crate::repositories::analytics::site::user_rollups::UserDailyRollupRow;
use crate::repositories::analytics::site::{
    SiteScope, code, distribution, kpis, series, session_costs, user_rollups,
};
use crate::util::time_range::TimeRange;

#[derive(Default)]
pub(super) struct AnalyticsUserData {
    pub kpis: SiteKpis,
    pub series: Vec<UsageBucket>,
    pub models: Vec<ModelDistributionRow>,
    pub code_series: Vec<CodeDayBucket>,
    pub code_totals: CodeTotals,
    pub daily: Vec<UserDailyRollupRow>,
    pub sessions: Vec<UserSessionCostRow>,
    pub permissions: PermissionGrantStats,
}

pub(super) async fn load_user_data(
    pool: &PgPool,
    user_id: &UserId,
    range: TimeRange,
    scope: &SiteScope,
    session_limit: i64,
) -> AnalyticsUserData {
    let (
        kpis_res,
        series_res,
        models_res,
        code_res,
        totals_res,
        daily_res,
        sessions_res,
        perms_res,
    ) = tokio::join!(
        kpis::get_site_kpis(pool, range, scope),
        series::list_daily_usage_series(pool, range, scope, SeriesBucket::Day),
        distribution::list_model_distribution(pool, range, scope),
        code::list_daily_code_series(pool, range, scope),
        code::get_code_totals(pool, range, scope),
        user_rollups::list_user_daily_rollups(pool, user_id, range),
        session_costs::list_user_session_costs(pool, user_id, session_limit),
        kpis::get_permission_grant_stats(pool, range, scope),
    );

    AnalyticsUserData {
        kpis: kpis_res.unwrap_or_else(|e| {
            tracing::warn!(error = %e, "get_site_kpis failed (user page)");
            SiteKpis::default()
        }),
        series: unwrap_or_empty(series_res, "list_daily_usage_series"),
        models: unwrap_or_empty(models_res, "list_model_distribution"),
        code_series: unwrap_or_empty(code_res, "list_daily_code_series"),
        code_totals: totals_res.unwrap_or_else(|e| {
            tracing::warn!(error = %e, "get_code_totals failed (user page)");
            CodeTotals::default()
        }),
        daily: unwrap_or_empty(daily_res, "list_user_daily_rollups"),
        sessions: unwrap_or_empty(sessions_res, "list_user_session_costs"),
        permissions: perms_res.unwrap_or_else(|e| {
            tracing::warn!(error = %e, "get_permission_grant_stats failed (user page)");
            PermissionGrantStats::default()
        }),
    }
}

fn unwrap_or_empty<T>(res: Result<Vec<T>, sqlx::Error>, what: &'static str) -> Vec<T> {
    res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, query = what, "user analytics query failed");
        Vec::new()
    })
}
