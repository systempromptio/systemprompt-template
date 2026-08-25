//! `repositories::config::plan_yaml_loader` — plans.yaml -> plans,
//! organizations, and projected organization rules.

use systemprompt_web_admin::authz::organization::organization_rule_type;
use systemprompt_web_admin::repositories::config::plan_yaml_loader::load_plans_from_yaml;

use crate::fixtures::{insert_acl_entity, unique, write_services_file};
use crate::tempdb::TempDb;

fn plans_yaml(plan: &str, org: &str, grant_entity: &str) -> String {
    format!(
        "plans:\n  \
           - id: {plan}\n    \
             name: Test Plan\n    \
             seat_limit: 25\n    \
             monthly_cost_cap_usd: 2500\n    \
             monthly_price_usd: 4000\n    \
             grants:\n      \
               - entity_type: marketplace\n        \
                 entity_id: {grant_entity}\n\
         organizations:\n  \
           - slug: {org}\n    \
             name: Test Org\n    \
             plan: {plan}\n    \
             email_domains: [test.example]\n"
    )
}

async fn org_rule_values(pool: &sqlx::PgPool, slug: &str) -> Vec<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT entity_id FROM access_control_rules
         WHERE rule_type = $1 AND rule_value = $2 ORDER BY entity_id",
    )
    .bind(organization_rule_type().as_str())
    .bind(slug)
    .fetch_all(pool)
    .await
    .expect("read projected rules")
}

#[tokio::test]
async fn load_plans_from_yaml_is_a_no_op_when_the_file_is_absent() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");

    let report = load_plans_from_yaml(&db.pool, dir.path())
        .await
        .expect("missing plans.yaml is not an error");

    assert_eq!(report.plans_upserted, 0);
    assert_eq!(report.organizations_upserted, 0);
    assert_eq!(report.grants_projected, 0);

    db.cleanup().await;
}

#[tokio::test]
async fn load_plans_from_yaml_accepts_an_empty_file() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    write_services_file(dir.path(), "access-control/plans.yaml", "   \n");

    let report = load_plans_from_yaml(&db.pool, dir.path())
        .await
        .expect("an empty file parses as an empty document");

    assert_eq!(report.plans_upserted, 0);

    db.cleanup().await;
}

#[tokio::test]
async fn load_plans_from_yaml_stores_prices_as_microdollars() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let plan = unique("plan");
    let org = unique("org");
    insert_acl_entity(&db.pool, "marketplace", "test-marketplace", false).await;
    write_services_file(
        dir.path(),
        "access-control/plans.yaml",
        &plans_yaml(&plan, &org, "test-marketplace"),
    );

    let report = load_plans_from_yaml(&db.pool, dir.path())
        .await
        .expect("load plans");

    assert_eq!(report.plans_upserted, 1);
    assert_eq!(report.organizations_upserted, 1);
    assert_eq!(report.grants_projected, 1);

    let row = sqlx::query_as::<_, (i64, Option<i64>, Option<i32>)>(
        "SELECT monthly_price_microdollars, monthly_cost_cap_microdollars, seat_limit
         FROM plans WHERE id = $1",
    )
    .bind(&plan)
    .fetch_one(&*db.pool)
    .await
    .expect("read plan");

    assert_eq!(row.0, 4_000_000_000, "$4000 authored in dollars");
    assert_eq!(row.1, Some(2_500_000_000));
    assert_eq!(row.2, Some(25));

    db.cleanup().await;
}

#[tokio::test]
async fn load_plans_from_yaml_projects_grants_onto_the_organization_slug() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let plan = unique("plan");
    let org = unique("org");
    insert_acl_entity(&db.pool, "marketplace", "test-marketplace", false).await;
    write_services_file(
        dir.path(),
        "access-control/plans.yaml",
        &plans_yaml(&plan, &org, "test-marketplace"),
    );

    load_plans_from_yaml(&db.pool, dir.path())
        .await
        .expect("load plans");

    assert_eq!(
        org_rule_values(&db.pool, &org).await,
        vec!["test-marketplace".to_owned()]
    );
    let entity_source = sqlx::query_scalar::<_, String>(
        "SELECT source FROM access_control_entities
         WHERE entity_type = 'marketplace' AND entity_id = 'test-marketplace'",
    )
    .fetch_one(&*db.pool)
    .await
    .expect("catalog row still present");
    assert_eq!(
        entity_source, "test",
        "projection consumes the pre-registered catalog row; it never mints one"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn load_plans_from_yaml_retracts_grants_the_plan_no_longer_carries() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let plan = unique("plan");
    let org = unique("org");
    insert_acl_entity(&db.pool, "marketplace", "first-marketplace", false).await;
    insert_acl_entity(&db.pool, "marketplace", "second-marketplace", false).await;
    write_services_file(
        dir.path(),
        "access-control/plans.yaml",
        &plans_yaml(&plan, &org, "first-marketplace"),
    );
    load_plans_from_yaml(&db.pool, dir.path())
        .await
        .expect("first load");

    write_services_file(
        dir.path(),
        "access-control/plans.yaml",
        &plans_yaml(&plan, &org, "second-marketplace"),
    );
    load_plans_from_yaml(&db.pool, dir.path())
        .await
        .expect("second load");

    assert_eq!(
        org_rule_values(&db.pool, &org).await,
        vec!["second-marketplace".to_owned()]
    );

    db.cleanup().await;
}

#[tokio::test]
async fn load_plans_from_yaml_leaves_another_organizations_rules_alone() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let other_org = unique("org-other");
    insert_acl_entity(&db.pool, "marketplace", "shared-marketplace", false).await;
    sqlx::query(
        "INSERT INTO access_control_rules (entity_type, entity_id, rule_type, rule_value, access)
         VALUES ('marketplace', 'shared-marketplace', $1, $2, 'allow')",
    )
    .bind(organization_rule_type().as_str())
    .bind(&other_org)
    .execute(&*db.pool)
    .await
    .expect("seed another customer's rule");

    let plan = unique("plan");
    let org = unique("org");
    insert_acl_entity(&db.pool, "marketplace", "own-marketplace", false).await;
    write_services_file(
        dir.path(),
        "access-control/plans.yaml",
        &plans_yaml(&plan, &org, "own-marketplace"),
    );
    load_plans_from_yaml(&db.pool, dir.path())
        .await
        .expect("load plans");

    assert_eq!(
        org_rule_values(&db.pool, &other_org).await,
        vec!["shared-marketplace".to_owned()]
    );

    db.cleanup().await;
}

#[tokio::test]
async fn load_plans_from_yaml_rejects_an_organization_on_an_unknown_plan() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    write_services_file(
        dir.path(),
        "access-control/plans.yaml",
        "plans: []\norganizations:\n  - slug: orphan\n    name: Orphan\n    plan: nope\n",
    );

    let result = load_plans_from_yaml(&db.pool, dir.path()).await;

    let message = result.err().expect("unknown plan is an error").to_string();
    assert!(
        message.contains("unknown plan"),
        "the message must name the problem, got: {message}"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn load_plans_from_yaml_rejects_an_unknown_entity_type_in_a_grant() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let plan = unique("plan");
    let org = unique("org");
    write_services_file(
        dir.path(),
        "access-control/plans.yaml",
        &format!(
            "plans:\n  - id: {plan}\n    name: P\n    grants:\n      - entity_type: teapot\n        entity_id: x\norganizations:\n  - slug: {org}\n    name: O\n    plan: {plan}\n"
        ),
    );

    let result = load_plans_from_yaml(&db.pool, dir.path()).await;

    assert!(result.is_err(), "an unknown entity kind must not be stored");

    db.cleanup().await;
}

#[tokio::test]
async fn load_plans_from_yaml_defaults_status_and_platform() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let plan = unique("plan");
    let org = unique("org");
    insert_acl_entity(&db.pool, "marketplace", "test-marketplace", false).await;
    write_services_file(
        dir.path(),
        "access-control/plans.yaml",
        &plans_yaml(&plan, &org, "test-marketplace"),
    );

    load_plans_from_yaml(&db.pool, dir.path())
        .await
        .expect("load plans");

    let row = sqlx::query_as::<_, (String, bool, Vec<String>)>(
        "SELECT status, is_platform, email_domains FROM organizations WHERE slug = $1",
    )
    .bind(&org)
    .fetch_one(&*db.pool)
    .await
    .expect("read organization");

    assert_eq!(row.0, "active");
    assert!(!row.1, "an ordinary customer is never the platform tenant");
    assert_eq!(row.2, vec!["test.example".to_owned()]);

    db.cleanup().await;
}

#[tokio::test]
async fn load_plans_from_yaml_rejects_a_grant_with_no_catalog_row_and_writes_nothing() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let plan = unique("plan");
    let org = unique("org");
    write_services_file(
        dir.path(),
        "access-control/plans.yaml",
        &plans_yaml(&plan, &org, "no-such-marketplace"),
    );

    let result = load_plans_from_yaml(&db.pool, dir.path()).await;

    let message = result
        .err()
        .expect("an unregistered entity id is a typo, not a new catalog row")
        .to_string();
    assert!(
        message.contains("no-such-marketplace"),
        "the message must name the offending id, got: {message}"
    );

    let plans = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM plans WHERE id = $1")
        .bind(&plan)
        .fetch_one(&*db.pool)
        .await
        .expect("count plans");
    assert_eq!(plans, 0, "validation failed, so nothing was persisted");
    let minted = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM access_control_entities WHERE entity_id = 'no-such-marketplace'",
    )
    .fetch_one(&*db.pool)
    .await
    .expect("count entities");
    assert_eq!(minted, 0, "no phantom catalog row");

    db.cleanup().await;
}
