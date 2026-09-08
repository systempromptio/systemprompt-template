//! Everything the governance page reads, in one pass.
//!
//! The KPI strip spans both planes and both are read on every tab, because the
//! whole claim of the page is that the chain and the scanners are configured
//! separately and have to be read together. Only the paged body is tab-shaped.
//!
//! Each read degrades to an empty result with a warning rather than failing the
//! page: a governance surface that 500s because one rollup timed out tells an
//! operator nothing, and the tab they came for is usually still readable.

use sqlx::PgPool;

use crate::repositories::governance::decision_log::{
    DecisionFilter, DecisionLogRow, DecisionSort, DecisionStats, get_decision_stats,
    list_decision_policies, list_governance_decisions_paged,
};
use crate::repositories::governance::findings::{
    FindingFilter, SafetyFindingLogRow, SafetyStats, get_safety_stats, list_finding_categories,
    list_safety_findings_paged,
};
use crate::repositories::governance::hook_events::{
    RecentHookEvent, count_posttool_fired_24h, count_pretool_fired_24h, recent_hook_events,
};
use crate::repositories::governance::{list_top_actors, list_top_policies};
use crate::repositories::scope::SubjectScope;
use crate::types::{TopActor, TopPolicy};
use crate::util::time_range::TimeRange;

use super::GovernanceTab;

const RANK_LIMIT: i64 = 10;
const HOOK_LIMIT: i64 = 50;

// Why: What one render of the page needs from the database.
pub(super) struct DashboardGovernanceData {
    pub(super) stats: DecisionStats,
    pub(super) safety: SafetyStats,
    pub(super) policies: Vec<String>,
    pub(super) categories: Vec<String>,
    pub(super) decisions: Vec<DecisionLogRow>,
    pub(super) decision_total: i64,
    pub(super) findings: Vec<SafetyFindingLogRow>,
    pub(super) finding_total: i64,
    pub(super) hooks: Vec<RecentHookEvent>,
    pub(super) pretool_24h: i64,
    pub(super) posttool_24h: i64,
    pub(super) top_policies: Vec<TopPolicy>,
    pub(super) top_actors: Vec<TopActor>,
}

// Why: The parameters one load is bound by.
pub(super) struct GovernanceRead<'a> {
    pub(super) tab: GovernanceTab,
    pub(super) range: TimeRange,
    pub(super) scope: &'a SubjectScope,
    pub(super) decision_filter: &'a DecisionFilter,
    pub(super) finding_filter: &'a FindingFilter,
    pub(super) sort: DecisionSort,
    pub(super) page_size: i64,
    pub(super) offset: i64,
}

pub(super) async fn load(pool: &PgPool, read: GovernanceRead<'_>) -> DashboardGovernanceData {
    let stats = get_decision_stats(pool, read.range, read.scope)
        .await
        .unwrap_or_else(|e| warn_default("governance decision stats", &e));
    let safety = get_safety_stats(pool, read.range, read.scope)
        .await
        .unwrap_or_else(|e| warn_default("safety finding stats", &e));
    let policies = list_decision_policies(pool, read.range, read.scope)
        .await
        .unwrap_or_default();
    let categories = list_finding_categories(pool, read.range, read.scope)
        .await
        .unwrap_or_default();

    let (decisions, decision_total) = if read.tab == GovernanceTab::Decisions {
        list_governance_decisions_paged(
            pool,
            read.range,
            read.scope,
            read.decision_filter,
            read.sort,
            read.page_size,
            read.offset,
        )
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "governance decision log failed");
            (Vec::new(), 0)
        })
    } else {
        (Vec::new(), stats.evaluated)
    };

    let (findings, finding_total) = if read.tab == GovernanceTab::Safety {
        list_safety_findings_paged(
            pool,
            read.range,
            read.scope,
            read.finding_filter,
            read.page_size,
            read.offset,
        )
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "safety finding log failed");
            (Vec::new(), 0)
        })
    } else {
        (Vec::new(), safety.findings)
    };

    let window_seconds = (read.range.to - read.range.from).num_seconds().max(1);
    let hooks = load_hooks(pool, read.tab, window_seconds).await;

    DashboardGovernanceData {
        stats,
        safety,
        policies,
        categories,
        decisions,
        decision_total,
        findings,
        finding_total,
        hooks: hooks.events,
        pretool_24h: hooks.pretool_24h,
        posttool_24h: hooks.posttool_24h,
        top_policies: hooks.top_policies,
        top_actors: hooks.top_actors,
    }
}

#[derive(Default)]
struct HookData {
    events: Vec<RecentHookEvent>,
    pretool_24h: i64,
    posttool_24h: i64,
    top_policies: Vec<TopPolicy>,
    top_actors: Vec<TopActor>,
}

// Why: five reads nothing else on the page needs, so they run only when the
// hooks tab asked for them. Loading them on every tab would put the rankings'
// two GROUP BYs on the critical path of the decisions log.
async fn load_hooks(pool: &PgPool, tab: GovernanceTab, window_seconds: i64) -> HookData {
    if tab != GovernanceTab::Hooks {
        return HookData::default();
    }
    HookData {
        events: recent_hook_events(pool, HOOK_LIMIT)
            .await
            .unwrap_or_default(),
        pretool_24h: count_pretool_fired_24h(pool).await.unwrap_or(0),
        posttool_24h: count_posttool_fired_24h(pool).await.unwrap_or(0),
        top_policies: list_top_policies(pool, window_seconds, RANK_LIMIT)
            .await
            .unwrap_or_default(),
        top_actors: list_top_actors(pool, window_seconds, RANK_LIMIT)
            .await
            .unwrap_or_default(),
    }
}

fn warn_default<T: Default>(what: &str, error: &sqlx::Error) -> T {
    tracing::warn!(error = %error, surface = what, "governance read failed");
    T::default()
}
