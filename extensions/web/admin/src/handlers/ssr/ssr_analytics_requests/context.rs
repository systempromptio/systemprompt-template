//! Typed view-model structs for the Inference Requests (`analytics-requests`)
//! page. Mirrors every `{{field}}` / `{{#each}}` / `{{#if}}` referenced by
//! `storage/files/admin/templates/analytics-requests.hbs`.

use serde::Serialize;
use systemprompt::identifiers::{AiRequestId, SessionId, TraceId, UserId};

#[derive(Debug, Serialize)]
pub(super) struct AnalyticsRequestsPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub time_range: TimeRangeView,
    pub stats: RequestStatsView,
    pub histogram: Vec<LatencyBucketView>,
    pub histogram_max: i64,
    pub cost_series: Vec<CostBucketView>,
    pub cost_max: i64,
    pub rows: Vec<RequestRowView>,
    pub has_rows: bool,
    pub total_count: i64,
    pub pagination: PaginationView,
    pub search_query: String,
    pub filters: FiltersView,
    pub has_active_filters: bool,
    pub clear_url: String,
    pub base_url: &'static str,
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
pub(super) struct LatencyBucketView {
    pub label: String,
    pub count: i64,
    pub upper_bound_ms: Option<f64>,
}

#[derive(Debug, Serialize)]
pub(super) struct CostBucketView {
    pub bucket_index: i32,
    pub bucket_start: String,
    pub cost_microdollars: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct RequestRowView {
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
    /// Only meaningful when the requested window had to be widened; the
    /// template gates its notice banner on `{{#if time_range.auto_widened}}`,
    /// so an *absent* key (not `null`) must mean "not widened".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_widened: Option<&'static str>,
}

#[derive(Debug, Serialize)]
pub(super) struct FiltersView {
    pub model: Option<String>,
    pub provider: Option<String>,
    pub status: Option<String>,
    pub options: FilterOptionsView,
}

#[derive(Debug, Serialize)]
pub(super) struct FilterOptionsView {
    pub models: Vec<String>,
    pub providers: Vec<String>,
    pub statuses: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct PaginationView {
    pub current_page: i64,
    pub total_pages: i64,
    pub has_prev: bool,
    pub has_next: bool,
    pub prev_url: Option<String>,
    pub next_url: Option<String>,
}
