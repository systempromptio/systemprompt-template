//! Template context types for the enterprise console.

use serde::Serialize;
use systemprompt::identifiers::UserId;

// Why: One customer, as the enterprise list and the detail header show them.
//
// Money stays in microdollars all the way to the template, where `formatUsd`
// renders it. Rounding to dollars in Rust and again in Handlebars is how a
// total stops matching the rows above it.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct EnterpriseView {
    pub slug: String,
    pub name: String,
    pub plan_id: Option<String>,
    pub plan_name: Option<String>,
    pub status: String,
    pub is_active: bool,
    // Why: The operator's own tenant. Shown, because its spend is real, but it is
    // not a customer and carries no licence revenue.
    pub is_platform: bool,
    pub seats_used: i64,
    pub seat_limit: Option<i32>,
    // Why: Whether a limit applies at all. Separate from `seats_pct` because a
    // template cannot tell "no limit" from "0% used" — both are falsy — and
    // an empty enterprise on a capped plan must not read as unlimited.
    pub has_seat_limit: bool,
    pub seats_pct: i64,
    pub departments: i64,
    pub entitlements: i64,
    pub requests_30d: i64,
    pub tokens_30d: i64,
    pub cost_30d_microdollars: i64,
    pub cost_mtd_microdollars: i64,
    pub revenue_microdollars: i64,
    pub margin_microdollars: i64,
    pub margin_positive: bool,
    // Why: Whether the plan caps spend at all — the same "falsy zero" trap as
    // `has_seat_limit`.
    pub has_budget: bool,
    pub budget_pct: i64,
    pub budget_state: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EnterprisesPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub enterprises: Vec<EnterpriseView>,
    pub total_enterprises: i64,
    pub total_seats: i64,
    pub total_departments: i64,
    pub total_requests_30d: i64,
    pub total_revenue_microdollars: i64,
    pub total_cost_microdollars: i64,
    pub total_margin_microdollars: i64,
    // Why: Modifier class for the margin tile: empty when the portfolio is in
    // profit. Decided here rather than in the template, so "what counts as
    // bad" is one rule rather than one per surface that renders money.
    pub margin_variant: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EnterpriseMemberView {
    pub user_id: UserId,
    pub email: String,
    pub display_name: Option<String>,
    pub org_role: String,
    pub department: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EnterpriseDepartmentView {
    pub id: String,
    pub name: String,
    pub description: String,
    pub member_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EnterpriseEntitlementView {
    pub entity_type: String,
    pub entity_id: String,
    pub access: String,
    pub allowed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EnterpriseModelUsageView {
    pub model: String,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub share_pct: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EnterpriseDetailPageData {
    pub page: &'static str,
    pub title: String,
    pub enterprise: EnterpriseView,
    pub members: Vec<EnterpriseMemberView>,
    pub departments: Vec<EnterpriseDepartmentView>,
    pub entitlements: Vec<EnterpriseEntitlementView>,
    pub models: Vec<EnterpriseModelUsageView>,
    pub margin_variant: &'static str,
}
