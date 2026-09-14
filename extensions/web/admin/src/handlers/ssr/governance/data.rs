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

use crate::repositories::governance::decision_calls::{DecisionCallRow, list_decision_calls_paged};
use crate::repositories::governance::decision_log::{
    DecisionFilter, DecisionSort, DecisionStats, PolicyCount, get_decision_stats,
    list_decision_policies, list_decision_policy_counts,
};
use crate::repositories::governance::findings::{
    FindingFilter, SafetyFindingLogRow, SafetyStats, get_safety_stats, list_finding_categories,
    list_safety_findings_paged,
};
use crate::repositories::governance::hook_events::{
    RecentHookEvent, count_posttool_fired_24h, count_pretool_fired_24h, recent_hook_events,
};
use crate::repositories::governance::{
    DecisionPage, PageSlice, list_top_actors, list_top_policies,
};
use crate::repositories::scope::SubjectScope;
use crate::types::{TopActor, TopPolicy};
use crate::util::time_range::TimeRange;

use super::GovernanceTab;

const RANK_LIMIT: i64 = 10;
const HOOK_LIMIT: i64 = 50;

// Why: the band is a summary, not a second table. Ten is what fits above the
// log without pushing it off the screen; the "see all" link carries the rest.
const ATTENTION_LIMIT: i64 = 10;

// Why: What one render of the page needs from the database.
pub(super) struct GovernanceData {
    pub(super) stats: DecisionStats,
    pub(super) safety: SafetyStats,
    pub(super) policies: Vec<String>,
    pub(super) categories: Vec<String>,
    pub(super) decisions: Vec<DecisionCallRow>,
    pub(super) attention: Vec<DecisionCallRow>,
    pub(super) policy_counts: Vec<PolicyCount>,
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

impl GovernanceRead<'_> {
    const fn slice(&self) -> PageSlice {
        PageSlice {
            limit: self.page_size,
            offset: self.offset,
        }
    }
}

// Why: the band that puts the denials and the warnings above the log rather
// than somewhere inside twenty pages of allows. It is skipped when the log is
// already narrowed to them — the table would then be the band, repeated — and
// when the window is clean, because a band that is empty every day is a band
// nobody reads.
async fn load_attention(
    pool: &PgPool,
    read: &GovernanceRead<'_>,
    attention_calls: i64,
) -> Vec<DecisionCallRow> {
    if read.tab != GovernanceTab::Decisions
        || read.decision_filter.attention
        || attention_calls <= 0
    {
        return Vec::new();
    }
    let filter = DecisionFilter {
        attention: true,
        ..read.decision_filter.clone()
    };
    match list_decision_calls_paged(
        pool,
        read.range,
        read.scope,
        &filter,
        DecisionPage {
            sort: read.sort,
            slice: PageSlice::first(ATTENTION_LIMIT),
        },
    )
    .await
    {
        Ok((rows, _)) => rows,
        Err(e) => {
            tracing::warn!(error = %e, "governance attention band failed");
            Vec::new()
        },
    }
}

pub(super) async fn load(pool: &PgPool, read: GovernanceRead<'_>) -> GovernanceData {
    let stats = get_decision_stats(pool, read.range, read.scope)
        .await
        .unwrap_or_else(|e| warn_default("governance decision stats", &e));
    let safety = get_safety_stats(pool, read.range, read.scope)
        .await
        .unwrap_or_else(|e| warn_default("safety finding stats", &e));
    let policies = list_decision_policies(pool, read.range, read.scope)
        .await
        .unwrap_or_else(|e| warn_default("decision policies", &e));
    let categories = list_finding_categories(pool, read.range, read.scope)
        .await
        .unwrap_or_else(|e| warn_default("finding categories", &e));
    let policy_counts = list_decision_policy_counts(pool, read.range, read.scope)
        .await
        .unwrap_or_else(|e| warn_default("decision policy counts", &e));

    let (decisions, decision_total) = if read.tab == GovernanceTab::Decisions {
        list_decision_calls_paged(
            pool,
            read.range,
            read.scope,
            read.decision_filter,
            DecisionPage {
                sort: read.sort,
                slice: read.slice(),
            },
        )
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "governance decision log failed");
            (Vec::new(), 0)
        })
    } else {
        (Vec::new(), stats.calls)
    };

    let (findings, finding_total) = if read.tab == GovernanceTab::Safety {
        list_safety_findings_paged(
            pool,
            read.range,
            read.scope,
            read.finding_filter,
            read.slice(),
        )
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "safety finding log failed");
            (Vec::new(), 0)
        })
    } else {
        (Vec::new(), safety.findings)
    };

    let attention = load_attention(pool, &read, stats.attention_calls).await;

    let window_seconds = (read.range.to - read.range.from).num_seconds().max(1);
    let hooks = load_hooks(pool, read.tab, window_seconds).await;

    GovernanceData {
        stats,
        safety,
        policies,
        policy_counts,
        attention,
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
            .inspect_err(|e| tracing::warn!(error = %e, surface = "recent hook events", "governance read failed"))
            .unwrap_or_default(),
        pretool_24h: count_pretool_fired_24h(pool).await.unwrap_or(0),
        posttool_24h: count_posttool_fired_24h(pool).await.unwrap_or(0),
        top_policies: list_top_policies(pool, window_seconds, RANK_LIMIT)
            .await
            .unwrap_or_else(|e| warn_default("top policies", &e)),
        top_actors: list_top_actors(pool, window_seconds, RANK_LIMIT)
            .await
            .unwrap_or_else(|e| warn_default("top actors", &e)),
    }
}

fn warn_default<T: Default>(what: &str, error: &impl std::fmt::Display) -> T {
    tracing::warn!(error = %error, surface = what, "governance read failed");
    T::default()
}
