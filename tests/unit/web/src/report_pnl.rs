//! The customer report's no-cost guarantee.
//!
//! The customer report is a document that leaves the building. The guarantee
//! that it carries no internal cost lives in the SQL, not in the template or
//! the view-model — so it is asserted against the source of the queries.

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
