//! Typed view-model for the customer-facing month-end usage report.
//!
//! There is deliberately no cost, margin, or unit-economics field on this
//! struct. `plan_price_microdollars` is what the customer is billed — a
//! contracted number they already know — and it is the only money on the page.
//! The repository behind it selects no cost column either, so the two halves of
//! the guarantee are independent.

use serde::Serialize;

use crate::util::month_range::MonthOption;

#[derive(Debug, Serialize)]
pub(super) struct ReportCustomerContext {
    pub page: &'static str,
    pub title: String,
    pub org_name: String,
    pub org_slug: String,
    pub plan_name: Option<String>,
    pub plan_price_microdollars: i64,
    pub has_price: bool,
    pub month_key: String,
    pub month_label: String,
    pub month_complete: bool,
    pub months: Vec<MonthOption>,
    pub prev_url: String,
    pub next_url: Option<String>,
    pub has_next: bool,
    pub base_url: &'static str,
    pub generated_at: String,
    // Why: The org switcher only renders for a platform admin; a customer-side
    // administrator sees their own organization and no way to name another.
    pub is_platform_admin: bool,
    pub org_options: Vec<OrgOption>,
    pub summary: CustomerSummaryView,
    // Why: Handlebars reads an empty array as falsy, so each table gates on the
    // collection itself rather than on a parallel `has_` flag that could
    // disagree with it.
    pub users: Vec<UserUsageView>,
    pub departments: Vec<DepartmentUsageView>,
    pub models: Vec<ModelUsageView>,
}

#[derive(Debug, Serialize)]
pub(super) struct OrgOption {
    pub slug: String,
    pub name: String,
    pub selected: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct CustomerSummaryView {
    pub seats_used: i64,
    pub seat_limit: Option<i32>,
    pub has_seat_limit: bool,
    pub seats_display: String,
    pub active_users: i64,
    pub requests: i64,
    pub total_tokens: i64,
    pub total_tokens_display: String,
    pub input_tokens: i64,
    pub input_tokens_display: String,
    pub output_tokens: i64,
    pub output_tokens_display: String,
    pub cache_read_tokens: i64,
    pub cache_read_tokens_display: String,
    pub error_count: i64,
    pub success_rate_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct UserUsageView {
    pub display_name: String,
    pub email: String,
    pub department: String,
    pub requests: i64,
    pub input_tokens_display: String,
    pub output_tokens_display: String,
    pub total_tokens: i64,
    pub total_tokens_display: String,
    pub distinct_models: i64,
    pub share_pct: i64,
    pub share_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct DepartmentUsageView {
    pub department: String,
    pub members: i64,
    pub requests: i64,
    pub input_tokens_display: String,
    pub output_tokens_display: String,
    pub total_tokens: i64,
    pub total_tokens_display: String,
    pub share_pct: i64,
    pub share_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ModelUsageView {
    pub provider: String,
    pub model: String,
    pub requests: i64,
    pub input_tokens_display: String,
    pub output_tokens_display: String,
    pub cache_read_tokens_display: String,
    pub total_tokens: i64,
    pub total_tokens_display: String,
    pub share_pct: i64,
    pub share_display: String,
}
