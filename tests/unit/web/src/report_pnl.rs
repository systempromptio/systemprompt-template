//! Derived figures on the internal month-end P&L, and the customer report's
//! no-cost guarantee.
//!
//! The margin arithmetic is trivial; what is worth pinning is the shape of the
//! *undefined* cases. A non-billed plan has no margin rate and an uncapped plan
//! has no budget percentage, and both must come back as `None` rather than as
//! zero — a zero renders as "0% margin" or "plenty of budget left", which are
//! both confident statements the data does not support.

use systemprompt_web_admin::repositories::reports::internal::OrganizationMonthPnl;

fn org(revenue: i64, cost: i64, seats: i64, cap: Option<i64>) -> OrganizationMonthPnl {
    OrganizationMonthPnl {
        id: "org-1".to_owned(),
        slug: "acme".to_owned(),
        name: "Acme".to_owned(),
        status: "active".to_owned(),
        is_platform: false,
        plan_name: Some("Enterprise".to_owned()),
        revenue_microdollars: revenue,
        cap_microdollars: cap,
        seat_limit: Some(50),
        seats_used: seats,
        active_users: seats,
        requests: 100,
        tokens: 1_000_000,
        cost_microdollars: cost,
    }
}

#[test]
fn margin_is_revenue_less_cost() {
    assert_eq!(org(1000, 400, 10, None).margin_microdollars(), 600);
}

#[test]
fn a_customer_costing_more_than_they_pay_has_a_negative_margin() {
    let o = org(1000, 2500, 10, None);
    assert_eq!(o.margin_microdollars(), -1500);
    assert_eq!(o.margin_pct(), Some(-150));
}

#[test]
fn margin_rate_is_a_percentage_of_revenue() {
    assert_eq!(org(1000, 250, 10, None).margin_pct(), Some(75));
}

#[test]
fn a_non_billed_plan_has_no_margin_rate() {
    assert_eq!(org(0, 500, 10, None).margin_pct(), None);
}

#[test]
fn an_uncapped_plan_has_no_budget_percentage() {
    assert_eq!(org(1000, 500, 10, None).budget_used_pct(), None);
    assert_eq!(org(1000, 500, 10, Some(0)).budget_used_pct(), None);
}

#[test]
fn budget_percentage_can_exceed_100() {
    assert_eq!(org(1000, 1500, 10, Some(1000)).budget_used_pct(), Some(150));
}

#[test]
fn cost_per_seat_does_not_divide_by_zero() {
    assert_eq!(org(1000, 500, 0, None).cost_per_seat_microdollars(), 0);
    assert_eq!(org(1000, 500, 10, None).cost_per_seat_microdollars(), 50);
}

// The customer report is a document that leaves the building. The guarantee
// that it carries no internal cost lives in the SQL, not in the template or
// the view-model — so it is asserted against the source of the queries.
#[test]
fn the_customer_repository_selects_no_cost_column() {
    let src = include_str!("../../../../extensions/web/admin/src/repositories/reports/customer.rs");
    let sql_only: String = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !sql_only.contains("cost_microdollars"),
        "the customer report must never read a cost column"
    );
}
