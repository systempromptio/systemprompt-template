//! Typed view-model structs for the Inference Requests (`analytics-requests`)
//! page. Mirrors every `{{field}}` / `{{#each}}` / `{{#if}}` referenced by
//! `storage/files/admin/templates/analytics-requests.hbs` and its tab partials.

use serde::Serialize;
use systemprompt::identifiers::{AiRequestId, SessionId, TraceId, UserId};

use crate::handlers::ssr::list_view::{Chip, Pagination, ScopeFilterView, SelectOptionView};
use crate::handlers::ssr::types::{BreadcrumbView, ChartView, HistogramView, SortHeaderView};

pub(super) use crate::handlers::ssr::types::TabLinkView;

// Why: the log is the page. The rollups stay as sibling tabs because they are
// the fastest way to pick a filter, but a reader who opens `/admin/requests`
// wants rows, not a chart, so Log is what an unrecognised tab falls back to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RequestsTab {
    Log,
    Overview,
    Models,
    Providers,
    Status,
}

impl RequestsTab {
    pub(super) fn from_query(raw: Option<&str>) -> Self {
        match raw {
            Some("overview") => Self::Overview,
            Some("models") => Self::Models,
            Some("providers") => Self::Providers,
            Some("status") => Self::Status,
            _ => Self::Log,
        }
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Log => "log",
            Self::Overview => "overview",
            Self::Models => "models",
            Self::Providers => "providers",
            Self::Status => "status",
        }
    }
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "one flag per tab is what the template branches on"
)]
#[derive(Debug, Serialize)]
pub(super) struct AnalyticsRequestsPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub time_range: TimeRangeView,
    pub tabs: Vec<TabLinkView>,
    pub is_overview: bool,
    pub is_breakdown: bool,
    pub is_log: bool,
    pub kpis: RequestKpiView,
    pub stats: RequestStatsView,
    pub histogram: HistogramView,
    pub traffic_chart: ChartView,
    pub cost_chart: ChartView,
    pub breakdown: BreakdownView,
    pub filters: FilterOptionsView,
    pub sort_headers: RequestsSortHeaders,
    pub rows: Vec<RequestListRowView>,
    pub has_rows: bool,
    pub rows_unavailable: bool,
    pub total_count: i64,
    pub row_count_label: String,
    pub pagination: Pagination,
    pub search_query: String,
    pub chips: Vec<Chip>,
    pub has_active_filters: bool,
    pub clear_url: String,
    pub csv_url: String,
    pub base_url: &'static str,
    pub scope_filter: ScopeFilterView,
}

// Why: the toolbar's five `<select>` facets, plus the hidden fields that keep
// the window and the sort when the form submits.
#[derive(Debug, Default, Serialize)]
pub(super) struct FilterOptionsView {
    pub models: Vec<SelectOptionView>,
    pub providers: Vec<SelectOptionView>,
    pub statuses: Vec<SelectOptionView>,
    pub tools: Vec<SelectOptionView>,
    pub hidden: Vec<crate::handlers::ssr::list_view::HiddenFieldView>,
}

// Why: named fields rather than a Vec so the template addresses each header by
// name and the column set is checked at compile time.
#[derive(Debug, Serialize)]
pub(super) struct RequestsSortHeaders {
    pub time: SortHeaderView,
    pub tokens: SortHeaderView,
    pub cost: SortHeaderView,
    pub latency: SortHeaderView,
}

// Why: every number is over the filtered set rather than the raw window, so
// the tiles name what the table below them holds.
#[derive(Debug, Default, Serialize)]
pub(super) struct RequestKpiView {
    pub total: i64,
    pub total_display: String,
    pub unattributed: i64,
    pub attribution_sub: String,
    pub cost_display: String,
    pub tokens_sub: String,
    pub error_rate_display: String,
    pub failed: i64,
    pub failed_sub: String,
    pub has_failures: bool,
    pub p95_display: String,
    pub latency_sub: String,
    pub rejected: i64,
    pub rejected_url: String,
    pub rejected_active: bool,
    pub denied: i64,
    pub tool_calls: i64,
    pub tool_calls_sub: String,
}

// Why: one shape for all three breakdown tabs, so Models, Providers, and
// Status cannot drift apart.
#[derive(Debug, Serialize)]
pub(super) struct BreakdownView {
    pub dimension_label: &'static str,
    pub caption: &'static str,
    pub rows: Vec<BreakdownRowView>,
    pub has_rows: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct BreakdownRowView {
    pub key: String,
    pub requests: i64,
    pub share_pct: i64,
    pub share_display: String,
    pub tokens_display: String,
    pub cost_display: String,
    pub p50_display: String,
    pub p95_display: String,
    pub error_count: i64,
    pub error_rate_display: String,
    pub has_errors: bool,
    pub filter_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct RequestStatsView {
    pub total: i64,
    pub error_count: i64,
    pub requests_per_minute: String,
    pub p50_latency_ms: i64,
    pub p95_latency_ms: i64,
    pub p99_latency_ms: i64,
    pub total_cost_display: String,
    pub error_rate_pct: String,
    pub denied_session_count: i64,
    pub denied_session_rate_pct: String,
}

#[derive(Debug, Serialize)]
pub(super) struct RequestListRowView {
    pub id: String,
    pub detail_url: String,
    pub request_id: AiRequestId,
    pub trace_id: Option<TraceId>,
    pub trace_id_short: Option<String>,
    pub session_id: Option<SessionId>,
    pub user_id: UserId,
    pub user_url: String,
    pub user_label: String,
    // Why: exclusive attribution, so a row belongs to exactly one project and
    // one group. A row with neither is shown as "Unattributed" rather than
    // blank — the bucket is a fact about the data, not a gap in the page.
    pub project_id: Option<String>,
    pub project_label: String,
    pub project_url: Option<String>,
    pub group_id: Option<String>,
    pub group_label: String,
    pub group_url: Option<String>,
    pub is_unattributed: bool,
    pub provider: String,
    pub model: String,
    pub has_model: bool,
    pub status: String,
    pub is_error: bool,
    pub is_rejected: bool,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub tokens_display: String,
    pub cost_microdollars: i64,
    pub cost_display: String,
    pub latency_ms: Option<i32>,
    pub latency_display: String,
    pub error_message: Option<String>,
    pub decision_count: i64,
    pub deny_count: i64,
    pub is_denied_preflight: bool,
    pub governance_display: String,
    pub tool_call_count: i64,
    pub created_at: String,
    pub created_at_time: String,
    pub created_at_day: String,
}

#[derive(Debug, Serialize)]
pub(super) struct TimeRangeView {
    pub preset: String,
    pub from: String,
    pub to: String,
    pub base_url: &'static str,
    pub query: String,
    // Why: Only meaningful when the requested window had to be widened; the
    // template gates its notice banner on `{{#if time_range.auto_widened}}`,
    // so an *absent* key (not `null`) must mean "not widened".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_widened: Option<&'static str>,
    // Why: the sibling of `auto_widened` for the other way a page can answer
    // for a window nobody asked for — the URL named one and it could not be
    // read. Widening is a helpful choice; this is a rejection, and the reader
    // is owed both.
    pub rejected: bool,
}
