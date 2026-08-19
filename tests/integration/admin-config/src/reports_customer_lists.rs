//! `repositories::reports::customer` — the per-user, per-department, and
//! per-model tables under the header figures.

use systemprompt_web_admin::repositories::reports::customer::{
    list_customer_month_departments, list_customer_month_models, list_customer_month_users,
};

use crate::fixtures::{
    RequestSeed, add_member, ancient_window, at, insert_org, insert_plan, insert_request,
    insert_user, set_department, unique,
};
use crate::tempdb::TempDb;

// One organization on a priced plan with two members, both of whom made a
// request inside the historical window, plus one request outside it.
struct Scenario {
    org: String,
    heavy: String,
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

    Scenario { org, heavy, light }
}

#[tokio::test]
async fn list_customer_month_users_orders_by_tokens_consumed() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let scenario = seed(&db.pool).await;
    let (from, to) = ancient_window();

    let rows = list_customer_month_users(&db.pool, &scenario.org, from, to)
        .await
        .expect("list users");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].email, format!("{}@example.test", scenario.heavy));
    assert_eq!(rows[0].requests, 2);
    assert_eq!(rows[0].distinct_models, 2);
    assert_eq!(rows[1].requests, 1);

    db.cleanup().await;
}

#[tokio::test]
async fn list_customer_month_users_folds_an_unset_department_into_default() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let scenario = seed(&db.pool).await;
    let (from, to) = ancient_window();

    let rows = list_customer_month_users(&db.pool, &scenario.org, from, to)
        .await
        .expect("list users");

    let light = rows
        .iter()
        .find(|r| r.email.starts_with(&scenario.light))
        .expect("the lighter user is listed");
    assert_eq!(
        light.department, "Default",
        "an unset department must not drop the row"
    );
    let heavy = rows
        .iter()
        .find(|r| r.email.starts_with(&scenario.heavy))
        .expect("the heavier user is listed");
    assert_eq!(heavy.department, "Sales");

    db.cleanup().await;
}

#[tokio::test]
async fn list_customer_month_users_omits_members_with_no_activity() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let scenario = seed(&db.pool).await;
    let idle = unique("u-idle");
    insert_user(&db.pool, &idle).await;
    add_member(&db.pool, &idle, &scenario.org).await;
    let (from, to) = ancient_window();

    let rows = list_customer_month_users(&db.pool, &scenario.org, from, to)
        .await
        .expect("list users");

    assert!(!rows.iter().any(|r| r.email.starts_with(&idle)));

    db.cleanup().await;
}

#[tokio::test]
async fn list_customer_month_departments_aggregates_members_and_tokens() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let scenario = seed(&db.pool).await;
    let (from, to) = ancient_window();

    let rows = list_customer_month_departments(&db.pool, &scenario.org, from, to)
        .await
        .expect("list departments");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].department, "Sales", "heaviest department leads");
    assert_eq!(rows[0].members, 1);
    assert_eq!(rows[0].requests, 2);
    assert_eq!(rows[1].department, "Default");

    db.cleanup().await;
}

#[tokio::test]
async fn list_customer_month_models_groups_by_provider_and_model() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let scenario = seed(&db.pool).await;
    let (from, to) = ancient_window();

    let rows = list_customer_month_models(&db.pool, &scenario.org, from, to)
        .await
        .expect("list models");

    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|r| r.provider == "anthropic"));
    let sonnet = rows
        .iter()
        .find(|r| r.model == "claude-sonnet-4-5-20250929")
        .expect("sonnet row");
    assert_eq!(sonnet.requests, 2, "both members used the default model");
    assert_eq!(sonnet.cache_read_tokens, 50);

    db.cleanup().await;
}

#[tokio::test]
async fn customer_month_lists_are_empty_for_an_organization_with_no_traffic() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let org = unique("org");
    insert_org(&db.pool, &org, None).await;
    let (from, to) = ancient_window();

    let users = list_customer_month_users(&db.pool, &org, from, to)
        .await
        .expect("list users");
    let departments = list_customer_month_departments(&db.pool, &org, from, to)
        .await
        .expect("list departments");
    let models = list_customer_month_models(&db.pool, &org, from, to)
        .await
        .expect("list models");

    assert!(users.is_empty());
    assert!(departments.is_empty());
    assert!(models.is_empty());

    db.cleanup().await;
}
