//! Typed view-model structs for the per-user analytics page
//! (`analytics-user`). Mirrors every field the template references — strict
//! Handlebars means an absent field is a 500, so the two move together.

use serde::Serialize;
use systemprompt::identifiers::{SessionId, UserId};

use crate::handlers::ssr::types::{DeltaView, PieView, SvgLineChartView};

#[derive(Debug, Serialize)]
pub(super) struct AnalyticsUserContext {
    pub page: &'static str,
    pub title: String,
    pub user_id: UserId,
    pub label: String,
    pub email: String,
    pub roles_display: String,
    pub time_range: UserTimeRange,

    pub kpis: UserKpiView,
    pub trend_chart: SvgLineChartView,
    pub model_pie: PieView,
    pub code_chart: SvgLineChartView,

    pub code_totals: UserCodeTotalsView,
    pub daily_rows: Vec<UserDailyRowView>,
    pub has_daily_rows: bool,
    pub session_rows: Vec<UserSessionRowView>,
    pub has_session_rows: bool,

    pub log_url: String,
    pub manage_url: String,
    pub dashboard_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct UserTimeRange {
    pub preset: String,
    pub from: String,
    pub to: String,
    pub base_url: String,
    pub query: String,
}

#[derive(Debug, Serialize)]
pub(super) struct UserKpiView {
    pub requests: i64,
    pub requests_delta: DeltaView,
    pub cost_display: String,
    pub cost_delta: DeltaView,
    pub tokens_display: String,
    pub tokens_delta: DeltaView,
    pub error_display: String,
    pub requests_per_day_display: String,
    pub grant_rate_display: String,
    pub grant_has_data: bool,
}

// Why: the two line counts come from different measurement frames (hook-
// observed tool input vs git diff stats) and are never subtracted from one
// another, so they are shown side by side and labeled.
#[expect(
    clippy::struct_field_names,
    reason = "every field is a preformatted display string; the shared suffix is \
              the contract with the template, which prints them verbatim"
)]
#[derive(Debug, Serialize)]
pub(super) struct UserCodeTotalsView {
    pub loc_added_ai_display: String,
    pub loc_removed_ai_display: String,
    pub committed_display: String,
    pub commits_display: String,
    pub edit_operations_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct UserDailyRowView {
    pub date_display: String,
    pub sessions: i64,
    pub prompts: i64,
    pub tool_uses: i64,
    pub requests: i64,
    pub loc_added_display: String,
    pub commits: i64,
    pub cost_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct UserSessionRowView {
    pub session_id: SessionId,
    pub model: String,
    pub cost_display: String,
    pub context_display: String,
    pub cache_read_display: String,
    pub tokens_display: String,
    pub updated_display: String,
    pub session_url: String,
}
