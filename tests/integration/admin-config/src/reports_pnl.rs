//! `repositories::reports::internal` — one row per organization for the month.
//!
//! The database arrives pre-seeded with a house organization and three demo
//! tenants, so every assertion here names its own organization rather than
//! counting rows.

use systemprompt_web_admin::repositories::reports::internal::{
    OrganizationMonthPnl, list_organization_month_pnl,
};

use crate::fixtures::{
    RequestSeed, add_member, ancient_window, at, insert_org, insert_org_with_status, insert_plan,
    insert_request, insert_user, unique,
};
use crate::tempdb::TempDb;

struct Scenario {
    org: String,
}

// One priced organization that spent 900_000 microdollars of a 4_000_000_000
// licence inside the historical window, plus one request outside it.
async fn seed(pool: &sqlx::PgPool) -> Scenario {
    let plan = unique("plan");
    let org = unique("org");
    let user = unique("u");
    insert_plan(pool, &plan, 4_000_000_000, Some(2_500_000_000), Some(25)).await;
    insert_org(pool, &org, Some(&plan)).await;
    insert_user(pool, &user).await;
    add_member(pool, &user, &org).await;

    let mut seed = RequestSeed::new("r-1", &user, at(2001, 3, 5, 9));
    seed.input_tokens = 1_000;
    seed.output_tokens = 200;
    seed.cost_microdollars = 600_000;
    insert_request(pool, &seed).await;

    let mut seed = RequestSeed::new("r-2", &user, at(2001, 3, 6, 9));
    seed.model = Some("claude-opus-4-5-20251101");
    seed.provider = Some("cerebras");
    seed.input_tokens = 100;
    seed.output_tokens = 20;
    seed.cost_microdollars = 300_000;
    insert_request(pool, &seed).await;

    let mut seed = RequestSeed::new("r-outside", &user, at(2001, 6, 1, 9));
    seed.cost_microdollars = 999_000_000;
    insert_request(pool, &seed).await;

    Scenario { org }
}

fn find_org<'a>(rows: &'a [OrganizationMonthPnl], id: &str) -> &'a OrganizationMonthPnl {
    rows.iter()
        .find(|r| r.id == id)
        .unwrap_or_else(|| panic!("organization {id} missing from the report"))
}

#[tokio::test]
async fn list_organization_month_pnl_reports_revenue_against_provider_cost() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let scenario = seed(&db.pool).await;
    let (from, to) = ancient_window();

    let rows = list_organization_month_pnl(&db.pool, from, to)
        .await
        .expect("list pnl");

    let row = find_org(&rows, &scenario.org);
    assert_eq!(row.revenue_microdollars, 4_000_000_000);
    assert_eq!(row.cost_microdollars, 900_000);
    assert_eq!(row.requests, 2);
    assert_eq!(row.tokens, 1_320);

    db.cleanup().await;
}

#[tokio::test]
async fn list_organization_month_pnl_includes_organizations_with_no_traffic() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let org = unique("org");
    insert_org(&db.pool, &org, None).await;
    let (from, to) = ancient_window();

    let rows = list_organization_month_pnl(&db.pool, from, to)
        .await
        .expect("list pnl");

    let row = find_org(&rows, &org);
    assert_eq!(row.requests, 0);
    assert_eq!(row.cost_microdollars, 0);
    assert!(row.plan_name.is_none());
    assert_eq!(
        row.margin_pct(),
        None,
        "margin is undefined on a non-billed plan, not zero"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn list_organization_month_pnl_carries_the_status_through() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let org = unique("org");
    insert_org_with_status(&db.pool, &org, None, "suspended").await;
    let (from, to) = ancient_window();

    let rows = list_organization_month_pnl(&db.pool, from, to)
        .await
        .expect("list pnl");

    assert_eq!(find_org(&rows, &org).status, "suspended");

    db.cleanup().await;
}

#[tokio::test]
async fn list_organization_month_pnl_sorts_the_platform_tenant_first() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let (from, to) = ancient_window();

    let rows = list_organization_month_pnl(&db.pool, from, to)
        .await
        .expect("list pnl");

    let platform_last = rows.iter().rposition(|r| r.is_platform);
    let customer_first = rows.iter().position(|r| !r.is_platform);
    if let (Some(platform_last), Some(customer_first)) = (platform_last, customer_first) {
        assert!(
            platform_last < customer_first,
            "the operator's own tenant leads the report"
        );
    }

    db.cleanup().await;
}

#[tokio::test]
async fn organization_month_pnl_derives_margin_from_revenue_and_cost() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let scenario = seed(&db.pool).await;
    let (from, to) = ancient_window();

    let rows = list_organization_month_pnl(&db.pool, from, to)
        .await
        .expect("list pnl");

    let row = find_org(&rows, &scenario.org);
    assert_eq!(row.margin_microdollars(), 4_000_000_000 - 900_000);
    assert_eq!(row.margin_pct(), Some(99));
    assert_eq!(row.cost_per_seat_microdollars(), 900_000, "one active seat");
    assert_eq!(row.budget_used_pct(), Some(0), "well inside a $2500 cap");

    db.cleanup().await;
}

#[tokio::test]
async fn organization_month_pnl_reports_no_budget_figure_without_a_cap() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let plan = unique("plan");
    let org = unique("org");
    insert_plan(&db.pool, &plan, 1_000_000, None, None).await;
    insert_org(&db.pool, &org, Some(&plan)).await;
    let (from, to) = ancient_window();

    let rows = list_organization_month_pnl(&db.pool, from, to)
        .await
        .expect("list pnl");

    let row = find_org(&rows, &org);
    assert_eq!(
        row.budget_used_pct(),
        None,
        "an uncapped plan must not render as 0% headroom"
    );
    assert_eq!(row.cost_per_seat_microdollars(), 0, "no seats, no divisor");
    assert!(row.seat_limit.is_none());

    db.cleanup().await;
}

#[tokio::test]
async fn organization_month_pnl_prefers_the_negotiated_seat_override() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let plan = unique("plan");
    let org = unique("org");
    insert_plan(&db.pool, &plan, 0, None, Some(25)).await;
    insert_org(&db.pool, &org, Some(&plan)).await;
    sqlx::query("UPDATE organizations SET seat_limit_override = 99 WHERE id = $1")
        .bind(&org)
        .execute(&*db.pool)
        .await
        .expect("set seat override");
    let (from, to) = ancient_window();

    let rows = list_organization_month_pnl(&db.pool, from, to)
        .await
        .expect("list pnl");

    assert_eq!(find_org(&rows, &org).seat_limit, Some(99));

    db.cleanup().await;
}
