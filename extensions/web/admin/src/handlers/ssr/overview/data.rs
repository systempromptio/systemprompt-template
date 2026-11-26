//! Loading the overview's widgets.
//!
//! Every widget keeps its own `Result` all the way to the view, which renders
//! the failed ones as a notice in place and draws the rest. The landing page
//! is where an operator finds out something is wrong; a page that blanks
//! because one aggregate could not be read is the one failure mode it must not
//! have.

use std::sync::Arc;

use sqlx::PgPool;

#[cfg(feature = "gateway")]
use crate::repositories::analytics::site::SiteScope;
#[cfg(feature = "gateway")]
use crate::repositories::analytics::site::models::{ModelStatsRow, list_model_stats};
#[cfg(feature = "gateway")]
use crate::repositories::overview::kpis::OverviewKpis;
use crate::repositories::overview::queues;
#[cfg(feature = "gateway")]
use crate::repositories::overview::queues::UsageAnomalyRow;
#[cfg(feature = "gateway")]
use crate::repositories::overview::scopes::ScopeCostRow;
#[cfg(feature = "gateway")]
use crate::repositories::overview::{kpis, scopes, series};
#[cfg(feature = "gateway")]
use crate::repositories::scope::{ScopeKind, SubjectScope};

#[cfg(feature = "gateway")]
use super::view::OverviewRange;

// Why: how many containers each leaderboard carries. Five is what fits beside
// the other board without either one scrolling.
#[cfg(feature = "gateway")]
pub(super) const TOP_SCOPES: i64 = 5;

// Why: how many models the usage board lists before pointing at the Models
// tab that owns the full table.
#[cfg(feature = "gateway")]
pub(super) const TOP_MODELS: usize = 5;

// Why: how many open anomalies the page lists before pointing at the page that
// owns them.
#[cfg(feature = "gateway")]
const TOP_ANOMALIES: i64 = 5;

pub(super) type Loaded<T> = Result<T, sqlx::Error>;

pub(super) struct OverviewData {
    pub pending_approvals: Loaded<i64>,
    #[cfg(feature = "gateway")]
    pub usage: UsageData,
}

// Why: the widgets the gateway member's request log supplies.
#[cfg(feature = "gateway")]
pub(super) struct UsageData {
    pub kpis: Loaded<OverviewKpis>,
    pub buckets: Loaded<Vec<i64>>,
    pub anomalies: Loaded<Vec<UsageAnomalyRow>>,
    pub projects: Loaded<Vec<ScopeCostRow>>,
    pub groups: Loaded<Vec<ScopeCostRow>>,
    pub models: Loaded<Vec<ModelStatsRow>>,
}

#[cfg(feature = "gateway")]
pub(super) async fn load_overview(pool: &Arc<PgPool>, range: OverviewRange) -> OverviewData {
    let (pending_approvals, usage) = tokio::join!(
        queues::count_pending_approvals(pool),
        load_usage(pool, range),
    );
    OverviewData {
        pending_approvals,
        usage,
    }
}

#[cfg(not(feature = "gateway"))]
pub(super) async fn load_overview(pool: &Arc<PgPool>) -> OverviewData {
    OverviewData {
        pending_approvals: queues::count_pending_approvals(pool).await,
    }
}

#[cfg(feature = "gateway")]
async fn load_usage(pool: &Arc<PgPool>, range: OverviewRange) -> UsageData {
    let window = range.time_range();
    // Why: the overview is instance-wide, so the model board reads the same
    // unscoped window every tile above it reads.
    let scope = SiteScope::new(SubjectScope::All);
    let (kpis, buckets, anomalies, projects, groups, models) = tokio::join!(
        kpis::get_overview_kpis(pool, window),
        series::list_request_buckets(pool, window),
        queues::list_open_anomalies(pool, TOP_ANOMALIES),
        scopes::list_top_scopes_by_cost(pool, ScopeKind::Project, window, TOP_SCOPES),
        scopes::list_top_scopes_by_cost(pool, ScopeKind::Group, window, TOP_SCOPES),
        list_model_stats(pool, window, &scope),
    );
    UsageData {
        kpis,
        buckets,
        anomalies,
        projects,
        groups,
        models,
    }
}
