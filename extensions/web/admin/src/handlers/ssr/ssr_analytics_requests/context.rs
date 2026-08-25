//! Typed view-model structs for the Inference Requests (`analytics-requests`)
//! page. Mirrors every `{{field}}` / `{{#each}}` / `{{#if}}` referenced by
//! `storage/files/admin/templates/analytics-requests.hbs` and its tab partials.

use serde::Serialize;
use systemprompt::identifiers::{AiRequestId, SessionId, TraceId, UserId};

use crate::handlers::ssr::list_view::Pagination;
use crate::handlers::ssr::types::{ChartView, HistogramView};

// Why: each tab is its own GET so it can be bookmarked, and so only the
// queries that tab renders ever run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RequestsTab {
    Overview,
    Models,
    Providers,
    Status,
    Log,
}

impl RequestsTab {
    // Why: anything unrecognised lands on Overview — a mistyped tab in a
    // shared link should still show the page rather than a 400.
    pub(super) fn from_query(raw: Option<&str>) -> Self {
        match raw {
            Some("models") => Self::Models,
            Some("providers") => Self::Providers,
            Some("status") => Self::Status,
            Some("log") => Self::Log,
            _ => Self::Overview,
        }
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Models => "models",
            Self::Providers => "providers",
            Self::Status => "status",
            Self::Log => "log",
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct AnalyticsRequestsPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub time_range: TimeRangeView,
    pub tabs: Vec<TabLinkView>,
    pub is_overview: bool,
    pub is_breakdown: bool,
    pub is_log: bool,
    pub stats: RequestStatsView,
    pub histogram: HistogramView,
    pub traffic_chart: ChartView,
    pub cost_chart: ChartView,
    pub breakdown: BreakdownView,
    pub rows: Vec<RequestListRowView>,
    pub has_rows: bool,
    pub total_count: i64,
    pub pagination: Pagination,
    pub search_query: String,
    pub chips: Vec<ChipView>,
    pub has_active_filters: bool,
    pub clear_url: String,
    pub base_url: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct TabLinkView {
    pub slug: &'static str,
    pub label: &'static str,
    pub href: String,
    pub is_active: bool,
    // Why: Shown as a count pill next to the label. Only the Log tab carries one:
    // on the other tabs the number the reader wants is already in the table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
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
pub(super) struct ChipView {
    pub group_label: &'static str,
    pub label: String,
    pub remove_url: String,
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
    pub request_id: AiRequestId,
    pub trace_id: Option<TraceId>,
    pub session_id: Option<SessionId>,
    pub user_id: UserId,
    pub user_label: String,
    pub provider: String,
    pub model: String,
    pub status: String,
    pub is_error: bool,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub tokens_total: i32,
    pub cost_microdollars: i64,
    pub cost_display: String,
    pub latency_ms: Option<i32>,
    pub error_message: Option<String>,
    pub decision_count: i64,
    pub deny_count: i64,
    pub is_denied_preflight: bool,
    pub tool_call_count: i64,
    pub created_at: String,
    pub created_at_local: String,
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
}
