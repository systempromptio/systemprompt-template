//! `repositories::reports::customer` — the header figures on a customer's
//! month.

use systemprompt_web_admin::repositories::reports::customer::find_customer_month_summary;

use crate::fixtures::{
    RequestSeed, add_member, ancient_window, at, insert_org, insert_plan, insert_request,
    insert_user, set_department, set_user_status, unique,
};
use crate::tempdb::TempDb;

// One organization on a priced plan with two members, both of whom made a
// request inside the historical window, plus one request outside it.
struct Scenario {
    org: String,
    light: String,
}

async fn seed(pool: &sqlx::PgPool) -> Scenario {
    let plan = unique("plan");
    let org = unique("org");
    let heavy = unique("u-heavy");
    let light = unique("u-light");
    insert_plan(pool, &plan, 4_000_000_000, Some(2_500_000_000), Some(25)).await;
    insert_org(pool, &org, Some(&plan)).await;
    for user in [&heavy, &light] {
        insert_user(pool, user).await;
        add_member(pool, user, &org).await;
    }
    set_department(pool, &heavy, "Sales").await;

    let mut seed = RequestSeed::new("r-heavy-1", &heavy, at(2001, 3, 5, 9));
    seed.input_tokens = 1_000;
    seed.output_tokens = 200;
    seed.cache_read_tokens = 50;
    seed.cost_microdollars = 900_000;
    insert_request(pool, &seed).await;

    let mut seed = RequestSeed::new("r-heavy-2", &heavy, at(2001, 3, 6, 9));
    seed.model = Some("claude-opus-4-5-20251101");
    seed.input_tokens = 500;
    seed.output_tokens = 100;
    seed.status = "failed";
    insert_request(pool, &seed).await;

    let mut seed = RequestSeed::new("r-light-1", &light, at(2001, 3, 7, 9));
    seed.input_tokens = 10;
    seed.output_tokens = 2;
    insert_request(pool, &seed).await;

    let mut seed = RequestSeed::new("r-outside", &light, at(2001, 5, 1, 9));
    seed.input_tokens = 999_999;
    insert_request(pool, &seed).await;

    Scenario { org, light }
}

#[tokio::test]
async fn find_customer_month_summary_returns_none_for_an_unknown_organization() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let (from, to) = ancient_window();

    let summary = find_customer_month_summary(&db.pool, &unique("nope"), from, to)
        .await
        .expect("query an absent organization");

    assert!(summary.is_none(), "find_ reports absence as None");

    db.cleanup().await;
}

#[tokio::test]
async fn find_customer_month_summary_reports_the_contracted_terms() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let scenario = seed(&db.pool).await;
    let (from, to) = ancient_window();

    let summary = find_customer_month_summary(&db.pool, &scenario.org, from, to)
        .await
        .expect("summary")
        .expect("the organization exists");

    assert_eq!(summary.org_id, scenario.org);
    assert_eq!(summary.price_microdollars, 4_000_000_000);
    assert_eq!(summary.seat_limit, Some(25));
    assert_eq!(summary.seats_used, 2);

    db.cleanup().await;
}

#[tokio::test]
async fn find_customer_month_summary_counts_only_requests_inside_the_window() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let scenario = seed(&db.pool).await;
    let (from, to) = ancient_window();

    let summary = find_customer_month_summary(&db.pool, &scenario.org, from, to)
        .await
        .expect("summary")
        .expect("the organization exists");

    assert_eq!(summary.requests, 3);
    assert_eq!(summary.input_tokens, 1_510);
    assert_eq!(summary.output_tokens, 302);
    assert_eq!(summary.cache_read_tokens, 50);
    assert_eq!(summary.total_tokens(), 1_812);

    db.cleanup().await;
}

#[tokio::test]
async fn find_customer_month_summary_counts_non_success_statuses_as_errors() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let scenario = seed(&db.pool).await;
    let (from, to) = ancient_window();

    let summary = find_customer_month_summary(&db.pool, &scenario.org, from, to)
        .await
        .expect("summary")
        .expect("the organization exists");

    assert_eq!(summary.error_count, 1, "one seeded request failed");
    assert_eq!(summary.active_users, 2);

    db.cleanup().await;
}

#[tokio::test]
async fn find_customer_month_summary_excludes_inactive_members_from_seats() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let scenario = seed(&db.pool).await;
    set_user_status(&db.pool, &scenario.light, "inactive").await;
    let (from, to) = ancient_window();

    let summary = find_customer_month_summary(&db.pool, &scenario.org, from, to)
        .await
        .expect("summary")
        .expect("the organization exists");

    assert_eq!(
        summary.seats_used, 1,
        "a seat is an active member, not a row in organization_members"
    );
    assert_eq!(
        summary.active_users, 2,
        "their requests still happened this month"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn find_customer_month_summary_reports_zeroes_for_a_quiet_month() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let org = unique("org");
    insert_org(&db.pool, &org, None).await;
    let (from, to) = ancient_window();

    let summary = find_customer_month_summary(&db.pool, &org, from, to)
        .await
        .expect("summary")
        .expect("the organization exists");

    assert_eq!(summary.requests, 0);
    assert_eq!(summary.seats_used, 0);
    assert_eq!(
        summary.price_microdollars, 0,
        "no plan means nothing billed"
    );
    assert!(summary.plan_name.is_none());

    db.cleanup().await;
}
