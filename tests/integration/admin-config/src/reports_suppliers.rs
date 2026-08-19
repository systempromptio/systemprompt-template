//! `repositories::reports::internal` — the supplier bill and the platform
//! cost trend.

use systemprompt_web_admin::repositories::reports::internal::{
    list_model_month_costs, list_platform_month_series, list_provider_month_costs,
};

use crate::fixtures::{
    RequestSeed, add_member, ancient_window, at, insert_org, insert_plan, insert_request,
    insert_user, unique,
};
use crate::tempdb::TempDb;

// The same month of traffic the P&L tests read, minus the identifiers: these
// aggregates are keyed by supplier, not by customer.
async fn seed(pool: &sqlx::PgPool) {
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
}

#[tokio::test]
async fn list_provider_month_costs_bills_each_upstream_separately() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed(&db.pool).await;
    let (from, to) = ancient_window();

    let rows = list_provider_month_costs(&db.pool, from, to)
        .await
        .expect("list provider costs");

    assert_eq!(rows.len(), 2, "only the two in-window requests are billed");
    assert_eq!(rows[0].key, "anthropic", "dearest supplier first");
    assert_eq!(rows[0].cost_microdollars, 600_000);
    assert_eq!(rows[0].tokens, 1_200);
    assert_eq!(rows[1].key, "cerebras");

    db.cleanup().await;
}

#[tokio::test]
async fn list_provider_month_costs_excludes_requests_that_never_reached_an_upstream() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    let mut rejected = RequestSeed::new("r-rejected", &user, at(2001, 3, 4, 9));
    rejected.status = "rejected";
    rejected.provider = None;
    rejected.model = None;
    insert_request(&db.pool, &rejected).await;
    let (from, to) = ancient_window();

    let providers = list_provider_month_costs(&db.pool, from, to)
        .await
        .expect("list provider costs");
    let models = list_model_month_costs(&db.pool, from, to)
        .await
        .expect("list model costs");

    assert!(providers.is_empty(), "a rejected request has no supplier");
    assert!(models.is_empty());

    db.cleanup().await;
}

#[tokio::test]
async fn list_model_month_costs_breaks_the_same_bill_down_by_model() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed(&db.pool).await;
    let (from, to) = ancient_window();

    let rows = list_model_month_costs(&db.pool, from, to)
        .await
        .expect("list model costs");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].key, "claude-sonnet-4-5-20250929");
    assert_eq!(rows[0].cost_microdollars, 600_000);
    assert_eq!(rows[1].key, "claude-opus-4-5-20251101");
    assert_eq!(rows[1].requests, 1);

    db.cleanup().await;
}

#[tokio::test]
async fn list_platform_month_series_returns_months_oldest_first() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    let now = chrono::Utc::now();
    let mut recent = RequestSeed::new("r-now", &user, now);
    recent.cost_microdollars = 12_345;
    insert_request(&db.pool, &recent).await;

    let rows = list_platform_month_series(&db.pool, 3)
        .await
        .expect("list month series");

    assert!(!rows.is_empty(), "this month has traffic");
    assert!(
        rows.windows(2)
            .all(|w| w[0].month_start <= w[1].month_start),
        "the trend chart reads left to right"
    );
    let current = rows.last().expect("at least one month");
    assert!(current.requests >= 1);
    assert!(current.cost_microdollars >= 12_345);

    db.cleanup().await;
}

#[tokio::test]
async fn list_platform_month_series_omits_months_before_the_horizon() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    insert_request(
        &db.pool,
        &RequestSeed::new("r-ancient", &user, at(2001, 3, 5, 9)),
    )
    .await;

    let rows = list_platform_month_series(&db.pool, 3)
        .await
        .expect("list month series");

    assert!(
        rows.iter()
            .all(|r| r.month_start.format("%Y").to_string() != "2001"),
        "a request from 2001 is outside a three-month horizon"
    );

    db.cleanup().await;
}
