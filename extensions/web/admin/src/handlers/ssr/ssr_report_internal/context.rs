//! Typed view-model for the internal month-end P&L report.
//!
//! Mirrors every field `storage/files/admin/templates/report-internal.hbs`
//! references. Handlebars runs in strict mode, so a field the template reads
//! must exist here even when its value is null.
//!
//! Money stays in microdollars all the way to the template, where `formatUsd`
//! renders it once. Rounding in Rust and again in Handlebars is how a total
//! stops matching the rows above it.

use serde::Serialize;

use crate::handlers::ssr::types::ChartView;
use crate::util::month_range::MonthOption;

#[derive(Debug, Serialize)]
pub(super) struct ReportInternalContext {
    pub page: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub month_key: String,
    pub month_label: String,
    // Why: False while the month is still running. The template turns this into a
    // banner, because a partial month's margin is not a result.
    pub month_complete: bool,
    pub months: Vec<MonthOption>,
    pub prev_url: String,
    pub next_url: Option<String>,
    pub has_next: bool,
    pub base_url: &'static str,
    // Why: Always null here. The shared month-selector partial preserves an
    // organization across a month change on the customer report, and
    // Handlebars runs in strict mode: the key has to exist even where the
    // page has no organization to carry.
    pub org_slug: Option<String>,
    pub generated_at: String,
    pub totals: InternalTotals,
    // Why: Handlebars reads an empty array as falsy, so each table gates on the
    // collection itself rather than on a parallel `has_` flag that could
    // disagree with it.
    pub organizations: Vec<OrgPnlView>,
    pub providers: Vec<SupplierView>,
    pub models: Vec<SupplierView>,
    pub trend: ChartView,
}

// Why: The portfolio line. Every figure is folded from the same organization
// rows the table prints, so the header and the body cannot disagree.
#[derive(Debug, Serialize)]
pub(super) struct InternalTotals {
    pub customers: i64,
    pub seats: i64,
    pub active_users: i64,
    pub requests: i64,
    pub tokens: i64,
    pub tokens_display: String,
    pub revenue_microdollars: i64,
    pub cost_microdollars: i64,
    pub margin_microdollars: i64,
    pub margin_positive: bool,
    pub margin_variant: &'static str,
    // Why: `None` when nothing was billed — a percentage of zero revenue is
    // undefined, not 0%.
    pub margin_pct: Option<i64>,
    pub margin_pct_display: String,
    pub cost_per_1m_display: String,
    pub cost_per_request_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct OrgPnlView {
    pub slug: String,
    pub name: String,
    pub plan_name: Option<String>,
    pub status: String,
    pub is_active: bool,
    pub is_platform: bool,
    pub seats_used: i64,
    pub seat_limit: Option<i32>,
    pub has_seat_limit: bool,
    pub active_users: i64,
    pub requests: i64,
    pub tokens: i64,
    pub tokens_display: String,
    pub revenue_microdollars: i64,
    pub cost_microdollars: i64,
    pub margin_microdollars: i64,
    pub margin_positive: bool,
    pub margin_pct_display: String,
    pub cost_per_seat_microdollars: i64,
    pub budget_pct: Option<i64>,
    pub has_budget: bool,
    pub budget_state: &'static str,
    pub detail_url: String,
    // Why: Deep-links to the customer-facing report for the same month, so an
    // operator can read exactly what the customer will read.
    pub customer_report_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SupplierView {
    pub key: String,
    pub requests: i64,
    pub tokens_display: String,
    pub cost_microdollars: i64,
    pub share_pct: i64,
    pub share_display: String,
}
