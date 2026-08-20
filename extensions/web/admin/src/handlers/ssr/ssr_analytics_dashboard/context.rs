//! Typed view-model structs for the site analytics dashboard
//! (`analytics-dashboard`). Mirrors every `{{field}}` / `{{#each}}` /
//! `{{#if}}` referenced by
//! `storage/files/admin/templates/analytics-dashboard.hbs`.

use serde::Serialize;
use systemprompt::identifiers::UserId;

use crate::handlers::ssr::list_view::Pagination;
use crate::handlers::ssr::types::{ChartView, MeterView, PieView};

// Why: each tab is its own GET so it can be bookmarked, and so only the
// queries that tab renders ever run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DashboardTab {
    /// KPIs, the two trend charts, the model pie, and the org spend meter.
    Overview,
    /// Top-users leaderboard and adoption stats.
    Usage,
    /// Seat utilisation and the wasted-seats table.
    Seats,
    /// Per-organization spend against soft/hard caps.
    Spend,
    /// Commit activity and AI line deltas — two measurement frames.
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
    pub volume_chart: ChartView,
    pub cost_chart: ChartView,
    pub model_pie: PieView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub own_meter: Option<MeterView>,

    pub leaderboard: LeaderboardView,
    pub permissions: PermissionStatsView,

    pub seat_summary: Vec<SeatSummaryView>,
    pub wasted_seats: Vec<WastedSeatView>,
    pub has_wasted_seats: bool,

    pub spend_meters: Vec<MeterView>,
    pub has_spend_meters: bool,
    pub latency_link: String,

    pub commit_chart: ChartView,
    pub loc_chart: ChartView,
    pub code_frames: Vec<CodeFrameView>,
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
    /// Platform admins only — org admins are locked to their own org.
    pub show_org_select: bool,
    pub org_options: Vec<SelectOptionView>,
    pub department_options: Vec<SelectOptionView>,
    pub bucket_links: Vec<BucketLinkView>,
    /// Hidden fields the filter form must carry so submitting it keeps the
    /// window and tab.
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
    /// Re-renders the dashboard scoped to this user.
    pub scope_url: String,
    /// The raw request log, pre-filtered.
    pub log_url: String,
    /// The user-management detail page.
    pub detail_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct PermissionStatsView {
    pub requests: i64,
    pub granted: i64,
    pub rate_display: String,
    pub has_data: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct SeatSummaryView {
    pub org_name: String,
    pub seats_used: i64,
    pub seat_limit_display: String,
    pub pct: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct WastedSeatView {
    pub user_id: UserId,
    pub label: String,
    pub email: String,
    pub department: String,
    pub org_name: String,
    pub last_request_display: String,
    pub detail_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct CodeFrameView {
    pub title: &'static str,
    pub value_display: String,
    pub caption: &'static str,
}
