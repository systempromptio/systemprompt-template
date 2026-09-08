//! Typed view models for the Overview tab and the page chrome it shares —
//! the KPI strip, the top-user leaderboard, the latency split, the anomaly
//! list, the code frames, the tab bar, the filter bar and the time range.
//!
//! Split from `context.rs` at the 300-line ceiling.

use serde::Serialize;

use systemprompt::identifiers::UserId;

use crate::handlers::ssr::list_view::{Pagination, ScopeFilterView};
use crate::handlers::ssr::types::{DeltaView, SparklineView};

#[derive(Debug, Serialize)]
pub(super) struct AnomalyRowView {
    pub metric: String,
    pub window_display: String,
    pub observed_display: String,
    pub baseline_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct FastSlowView {
    // Why: the threshold belongs in the KPI label, not a footnote — "Within
    // SLO" alone is a number nobody can check without knowing what the SLO is.
    pub within_label: String,
    pub breach_label: String,
    pub fast: i64,
    pub slow: i64,
    pub untimed: i64,
    pub threshold_display: String,
    pub breach_pct_display: String,
    pub p50_display: String,
    pub p95_display: String,
    pub has_data: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct SloOption {
    pub label: String,
    pub href: String,
    pub selected: bool,
}

// Why: every figure here is client-reported statusline data, and the template
// labels it so — it complements the gateway's own token counts, never
// replaces them.
// Why: the thinking card sits beside the cache card because both answer the
// same question about a token total -- what share of it was not the answer.
#[derive(Debug, Serialize)]
pub(super) struct ThinkingView {
    pub has_data: bool,
    pub reasoning_display: String,
    pub output_display: String,
    pub share_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionCostsView {
    pub has_data: bool,
    pub sessions: i64,
    pub cache_hit_display: String,
    pub cache_read_display: String,
    pub avg_context_display: String,
    pub max_context_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct DashboardTimeRange {
    pub preset: String,
    pub from: String,
    pub to: String,
    pub base_url: &'static str,
    pub query: String,
    // Why: carried from `TimeRange::rejected_bounds`; the shared
    // `components/time-range` partial raises its notice on this, and the
    // dashboard is the page most likely to be reached by a pasted link
    // carrying someone else's window.
    pub rejected: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct DashboardTabLink {
    pub slug: &'static str,
    pub label: &'static str,
    pub href: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct FiltersView {
    pub scope: ScopeFilterView,
    pub bucket_links: Vec<BucketLinkView>,
}

#[derive(Debug, Serialize)]
pub(super) struct BucketLinkView {
    pub label: &'static str,
    pub href: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct ScopeChipView {
    pub group_label: &'static str,
    pub label: String,
    pub remove_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct KpiStripView {
    pub requests: i64,
    pub error_display: String,
    pub cost_display: String,
    pub cost_per_request_display: String,
    pub weekly_active_users: i64,
    pub active_users: i64,
    pub requests_per_user_day_display: String,
    pub tokens_display: String,
    pub requests_delta: DeltaView,
    pub cost_delta: DeltaView,
    pub wau_delta: DeltaView,
    pub tokens_delta: DeltaView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_spark: Option<SparklineView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_spark: Option<SparklineView>,
}

#[derive(Debug, Serialize)]
pub(super) struct LeaderboardView {
    // Why: which column the rows are actually ordered by, so the header can
    // carry `aria-sort` on that one column and on no other. A table that tells
    // a screen reader every column is sortable when only four links change the
    // order is worse than one that says nothing.
    pub sorted_key: &'static str,
    pub rows: Vec<LeaderRowView>,
    pub has_rows: bool,
    pub sort_links: Vec<SortLinkView>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize)]
pub(super) struct SortLinkView {
    pub label: &'static str,
    pub href: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct LeaderRowView {
    pub user_id: UserId,
    pub label: String,
    pub groups_display: String,
    pub requests: i64,
    pub share_pct: i64,
    pub tokens_display: String,
    pub cost_display: String,
    pub requests_per_day_display: String,
    pub last_active_display: String,
    pub scope_url: String,
    pub log_url: String,
    pub detail_url: String,
    pub analytics_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct PermissionStatsView {
    pub requests: i64,
    pub granted: i64,
    pub rate_display: String,
    pub has_data: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct CodeFrameView {
    // Why: serialized as frame_title — the layout partial's `title=` hash
    // param shadows `title` even via `this.title` in nested each-blocks.
    #[serde(rename = "frame_title")]
    pub title: &'static str,
    pub value_display: String,
    pub caption: &'static str,
}
