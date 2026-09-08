//! Loading the overview's widgets.
//!
//! Every widget keeps its own `Result` all the way to the view, which renders
//! the failed ones as a notice in place and draws the rest. The landing page
//! is where an operator finds out something is wrong; a page that blanks
//! because one aggregate could not be read is the one failure mode it must not
//! have.

use std::sync::Arc;

use sqlx::PgPool;

use crate::repositories::analytics::site::SiteScope;
use crate::repositories::analytics::site::models::{ModelStatsRow, list_model_stats};
use crate::repositories::overview::kpis::OverviewKpis;
use crate::repositories::overview::queues::UsageAnomalyRow;
use crate::repositories::overview::scopes::ScopeCostRow;
use crate::repositories::overview::{kpis, queues, scopes, series};
use crate::repositories::scope::{ScopeKind, SubjectScope};

use super::view::OverviewRange;

// Why: how many containers each leaderboard carries. Five is what fits beside
// the other board without either one scrolling.
pub(super) const TOP_SCOPES: i64 = 5;

// Why: how many models the usage board lists before pointing at the Models
// tab that owns the full table.
pub(super) const TOP_MODELS: usize = 5;

// Why: how many open anomalies the page lists before pointing at the page that
// owns them.
const TOP_ANOMALIES: i64 = 5;

type Loaded<T> = Result<T, sqlx::Error>;

pub(super) struct OverviewData {
    pub kpis: Loaded<OverviewKpis>,
    pub buckets: Loaded<Vec<i64>>,
    pub pending_approvals: Loaded<i64>,
    pub anomalies: Loaded<Vec<UsageAnomalyRow>>,
    pub projects: Loaded<Vec<ScopeCostRow>>,
    pub groups: Loaded<Vec<ScopeCostRow>>,
    pub models: Loaded<Vec<ModelStatsRow>>,
}

pub(super) async fn load_overview(pool: &Arc<PgPool>, range: OverviewRange) -> OverviewData {
    let window = range.time_range();
    // Why: the overview is instance-wide, so the model board reads the same
    // unscoped window every tile above it reads.
    let scope = SiteScope::new(SubjectScope::All);
    let (kpis, buckets, pending_approvals, anomalies, projects, groups, models) = tokio::join!(
        kpis::get_overview_kpis(pool, window),
        series::list_request_buckets(pool, window),
        queues::count_pending_approvals(pool),
        queues::list_open_anomalies(pool, TOP_ANOMALIES),
        scopes::list_top_scopes_by_cost(pool, ScopeKind::Project, window, TOP_SCOPES),
        scopes::list_top_scopes_by_cost(pool, ScopeKind::Group, window, TOP_SCOPES),
        list_model_stats(pool, window, &scope),
    );

    OverviewData {
        kpis,
        buckets,
        pending_approvals,
        anomalies,
        projects,
        groups,
        models,
    }
}
