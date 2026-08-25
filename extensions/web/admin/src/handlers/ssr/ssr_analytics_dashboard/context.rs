//! Typed view-model structs for the site analytics dashboard
//! (`analytics-dashboard`). Mirrors every `{{field}}` / `{{#each}}` /
//! `{{#if}}` referenced by
//! `storage/files/admin/templates/analytics-dashboard.hbs`.

use serde::Serialize;

pub(super) use super::context_seats::{InactiveDayOption, SeatSummaryView, WastedSeatView};
use systemprompt::identifiers::UserId;

use crate::handlers::ssr::list_view::Pagination;
use crate::handlers::ssr::types::{
    DeltaView, MeterView, PieView, SparklineView, SvgLineChartView, SvgStackedChartView,
};

// Why: each tab is its own GET so it can be bookmarked, and so only the
// queries that tab renders ever run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DashboardTab {
    Overview,
    Usage,
    Seats,
    Spend,
    Code,
}

impl DashboardTab {
    // Why: anything unrecognised lands on Overview — a mistyped tab in a
    // shared link should still show the page rather than a 400.
    pub(super) fn from_query(raw: Option<&str>) -> Self {
        match raw {
            Some("usage") => Self::Usage,
            Some("seats") => Self::Seats,
            Some("spend") => Self::Spend,
            Some("code") => Self::Code,
            _ => Self::Overview,
        }
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Usage => "usage",
            Self::Seats => "seats",
            Self::Spend => "spend",
            Self::Code => "code",
        }
    }
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "one is_* flag per tab plus the template's has_* guards; handlebars branches on \
              flat booleans, so folding them into an enum would just move the bools into the \
              serializer"
)]
#[derive(Debug, Serialize)]
pub(super) struct AnalyticsDashboardContext {
    pub page: &'static str,
    pub title: String,
    pub time_range: DashboardTimeRange,
    pub tabs: Vec<DashboardTabLink>,
    pub is_overview: bool,
    pub is_usage: bool,
    pub is_seats: bool,
    pub is_spend: bool,
    pub is_code: bool,

    pub filters: FiltersView,
    pub chips: Vec<ScopeChipView>,
    pub has_active_filters: bool,
    pub clear_url: String,
    pub base_url: &'static str,

    pub kpis: KpiStripView,
    pub volume_chart: SvgLineChartView,
    pub cost_chart: SvgLineChartView,
    pub model_pie: PieView,
    pub model_cost_chart: SvgStackedChartView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub own_meter: Option<MeterView>,

    pub leaderboard: LeaderboardView,
    pub permissions: PermissionStatsView,

    pub seat_summary: Vec<SeatSummaryView>,
    pub wasted_seats: Vec<WastedSeatView>,
    pub has_wasted_seats: bool,
    // Why: The resolved inactivity window, so the copy states the window actually
    // queried instead of a hardcoded "30 days" that a `?inactive_days=` would
    // silently contradict.
    pub inactive_days: i32,
    pub inactive_day_options: Vec<InactiveDayOption>,
    pub slo_options: Vec<SloOption>,

    pub spend_meters: Vec<MeterView>,
    pub has_spend_meters: bool,
    pub latency_link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burndown: Option<SvgLineChartView>,
    // Why: Shown to a platform admin who has not picked an organization — a
    // burn-up against a cap only means something for one org.
    pub show_burndown_hint: bool,
    pub budget_warnings: Vec<BudgetWarningRowView>,
    pub has_budget_warnings: bool,
    pub anomalies: Vec<AnomalyRowView>,
    pub has_anomalies: bool,
    pub fast_slow: FastSlowView,

    pub session_costs: SessionCostsView,

    pub commit_chart: SvgLineChartView,
    pub loc_chart: SvgLineChartView,
    pub code_frames: Vec<CodeFrameView>,
}

#[derive(Debug, Serialize)]
pub(super) struct BudgetWarningRowView {
    pub org_name: String,
    pub kind_display: &'static str,
    pub month_display: String,
    pub threshold_display: String,
    pub spent_display: String,
    pub over_by_display: String,
    pub first_seen_display: String,
    pub last_seen_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct AnomalyRowView {
    pub metric: String,
    pub window_display: String,
    pub observed_display: String,
    pub baseline_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct FastSlowView {
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
    pub show_org_select: bool,
    pub org_options: Vec<SelectOptionView>,
    pub department_options: Vec<SelectOptionView>,
    pub bucket_links: Vec<BucketLinkView>,
    pub hidden: Vec<HiddenFieldView>,
}

#[derive(Debug, Serialize)]
pub(super) struct SelectOptionView {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct BucketLinkView {
    pub label: &'static str,
    pub href: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct HiddenFieldView {
    pub name: &'static str,
    pub value: String,
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
    pub wasted_seats: i64,
    pub wasted_seats_url: String,
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
    pub department: String,
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
