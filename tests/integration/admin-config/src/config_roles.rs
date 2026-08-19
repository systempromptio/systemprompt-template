//! `repositories::config::acl_yaml_loader` and `::acl_yaml_snapshot` — the
//! role-rule bootstrap and its inverse.
//!
//! `services/access-control/departments.yaml` is deliberately not exercised:
//! the shipped file declares an empty list, and `upsert_department` writes no
//! `org_id` while conflicting on a `name` key that
//! the organizations backfill replaced with `(org_id, name)`.

use std::path::Path;

use systemprompt_security::authz::{Access, EntityKind, RuleType};
use systemprompt_web_admin::authz::organization::organization_rule_type;
use systemprompt_web_admin::repositories::config::acl_yaml_loader::load_from_yaml;
use systemprompt_web_admin::repositories::config::acl_yaml_snapshot::render_yaml_snapshot;
use systemprompt_web_admin::repositories::config::gateway_acl;
use systemprompt_web_admin::repositories::config::plan_yaml_loader::load_plans_from_yaml;

use crate::fixtures::{insert_acl_entity, unique, write_services_file};
use crate::tempdb::TempDb;

fn plans_yaml(plan: &str, org: &str, grant_entity: &str) -> String {
    format!(
        "plans:\n  - id: {plan}\n    name: Test Plan\n    grants:\n      - entity_type: marketplace\n        entity_id: {grant_entity}\norganizations:\n  - slug: {org}\n    name: Test Org\n    plan: {plan}\n"
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
async fn load_from_yaml_reports_nothing_when_no_files_exist() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");

    let report = load_from_yaml(&db.pool, dir.path())
        .await
        .expect("an empty services tree is not an error");

    assert_eq!(report.departments_upserted, 0);
    assert_eq!(report.rules_upserted, 0);

    db.cleanup().await;
}

#[tokio::test]
async fn load_from_yaml_ingests_role_rules() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let entity = unique("mkt");
    write_services_file(
        dir.path(),
        "access-control/roles.yaml",
        &format!(
            "rules:\n  - entity_type: marketplace\n    entity_id: {entity}\n    access: allow\n    default_included: true\n    roles: [admin, user]\n"
        ),
    );

    let report = load_from_yaml(&db.pool, dir.path())
        .await
        .expect("load roles");

    assert!(
        report.rules_upserted >= 2,
        "one rule per named role, got {}",
        report.rules_upserted
    );
    let roles = sqlx::query_scalar::<_, String>(
        "SELECT rule_value FROM access_control_rules
         WHERE entity_id = $1 AND rule_type = 'role' ORDER BY rule_value",
    )
    .bind(&entity)
    .fetch_all(&*db.pool)
    .await
    .expect("read ingested rules");
    assert_eq!(roles, vec!["admin".to_owned(), "user".to_owned()]);

    db.cleanup().await;
}

#[tokio::test]
async fn load_from_yaml_rejects_a_malformed_roles_file() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    write_services_file(
        dir.path(),
        "access-control/roles.yaml",
        "rules: \"not a list\"\n",
    );

    let result = load_from_yaml(&db.pool, dir.path()).await;

    assert!(
        result.is_err(),
        "a broken bootstrap file must fail loudly rather than load nothing"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn render_yaml_snapshot_collapses_roles_onto_one_entity_entry() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let entity = unique("route");
    insert_acl_entity(&db.pool, EntityKind::GatewayRoute.as_str(), &entity, true).await;
    for role in ["admin", "developer"] {
        gateway_acl::upsert_rule(&db.pool, &entity, RuleType::ROLE, role, Access::Allow)
            .await
            .expect("upsert role rule");
    }

    let yaml = render_yaml_snapshot(&db.pool)
        .await
        .expect("render snapshot");

    let block = yaml
        .split("- entity_type:")
        .find(|chunk| chunk.contains(&entity))
        .unwrap_or_else(|| panic!("entity {entity} missing from snapshot:\n{yaml}"));
    assert!(block.contains("admin"), "block was: {block}");
    assert!(block.contains("developer"), "block was: {block}");
    assert!(block.contains("default_included: true"));

    db.cleanup().await;
}

#[tokio::test]
async fn render_yaml_snapshot_omits_non_role_rules() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let entity = unique("route");
    insert_acl_entity(&db.pool, EntityKind::GatewayRoute.as_str(), &entity, false).await;
    gateway_acl::upsert_rule(
        &db.pool,
        &entity,
        organization_rule_type(),
        "some-customer",
        Access::Allow,
    )
    .await
    .expect("upsert organization rule");

    let yaml = render_yaml_snapshot(&db.pool)
        .await
        .expect("render snapshot");

    assert!(
        !yaml.contains("some-customer"),
        "roles.yaml has no shape for an organization rule, so it must be dropped"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn render_yaml_snapshot_of_a_route_with_no_role_rules_omits_it() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let entity = unique("route");
    insert_acl_entity(&db.pool, EntityKind::GatewayRoute.as_str(), &entity, false).await;

    let yaml = render_yaml_snapshot(&db.pool)
        .await
        .expect("render snapshot");

    assert!(
        !yaml.contains(&entity),
        "the snapshot lists rules, not the catalog"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn load_plans_from_yaml_is_idempotent_across_two_boots() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let tmp = tempfile::tempdir().expect("temp services dir");
    let dir: &Path = tmp.path();
    let plan = unique("plan");
    let org = unique("org");
    write_services_file(
        dir,
        "access-control/plans.yaml",
        &plans_yaml(&plan, &org, "mkt"),
    );

    load_plans_from_yaml(&db.pool, dir)
        .await
        .expect("first boot");
    load_plans_from_yaml(&db.pool, dir)
        .await
        .expect("second boot");

    let plans = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM plans WHERE id = $1")
        .bind(&plan)
        .fetch_one(&*db.pool)
        .await
        .expect("count plans");
    assert_eq!(plans, 1);
    assert_eq!(
        org_rule_values(&db.pool, &org).await,
        vec!["mkt".to_owned()]
    );

    db.cleanup().await;
}
