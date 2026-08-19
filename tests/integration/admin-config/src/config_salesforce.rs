//! `repositories::config::salesforce_yaml_loader` — salesforce.yaml -> the
//! `rule_type = 'salesforce'` gate rows, closed entity defaults, and the
//! removal of the pre-gate role grant.

use systemprompt_web_admin::authz::salesforce::{
    SALESFORCE_LINKED_VALUE, salesforce_dimension, salesforce_rule_type,
};
use systemprompt_web_admin::repositories::config::salesforce_yaml_loader::load_salesforce_from_yaml;

use crate::fixtures::{insert_acl_entity, unique, write_services_file};
use crate::tempdb::TempDb;

fn salesforce_yaml(entity_type: &str, entity_id: &str) -> String {
    format!(
        "grants:\n  \
           - entity_type: {entity_type}\n    \
             entity_id: {entity_id}\n"
    )
}

async fn linked_rule_entities(pool: &sqlx::PgPool) -> Vec<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT entity_id FROM access_control_rules
         WHERE rule_type = $1 AND rule_value = $2 ORDER BY entity_id",
    )
    .bind(salesforce_rule_type().as_str())
    .bind(SALESFORCE_LINKED_VALUE)
    .fetch_all(pool)
    .await
    .expect("read projected rules")
}

#[test]
fn the_salesforce_dimension_sits_between_department_and_role() {
    let dim = salesforce_dimension();
    assert_eq!(dim.rule_type.as_str(), "salesforce");
    assert_eq!(dim.label, "Salesforce");
    assert!(
        dim.precedence > 100 && dim.precedence < 200,
        "must out-rank the role band and defer to department, got {}",
        dim.precedence
    );
}

#[tokio::test]
async fn load_salesforce_from_yaml_is_a_no_op_when_the_file_is_absent() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");

    let report = load_salesforce_from_yaml(&db.pool, dir.path())
        .await
        .expect("missing salesforce.yaml is not an error");

    assert_eq!(report.grants_projected, 0);

    db.cleanup().await;
}

#[tokio::test]
async fn load_salesforce_from_yaml_projects_a_closed_default_and_a_linked_rule() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let plugin = unique("sf-plugin");
    write_services_file(
        dir.path(),
        "access-control/salesforce.yaml",
        &salesforce_yaml("plugin", &plugin),
    );

    let report = load_salesforce_from_yaml(&db.pool, dir.path())
        .await
        .expect("load salesforce gate");

    assert_eq!(report.grants_projected, 1);
    assert_eq!(linked_rule_entities(&db.pool).await, vec![plugin.clone()]);

    let default_included = sqlx::query_scalar::<_, bool>(
        "SELECT default_included FROM access_control_entities
         WHERE entity_type = 'plugin' AND entity_id = $1",
    )
    .bind(&plugin)
    .fetch_one(&*db.pool)
    .await
    .expect("catalog row exists");
    assert!(
        !default_included,
        "an unlinked user falls through to the entity default, which must be closed"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn load_salesforce_from_yaml_forces_an_open_default_closed() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let plugin = unique("sf-plugin");
    insert_acl_entity(&db.pool, "plugin", &plugin, true).await;
    write_services_file(
        dir.path(),
        "access-control/salesforce.yaml",
        &salesforce_yaml("plugin", &plugin),
    );

    load_salesforce_from_yaml(&db.pool, dir.path())
        .await
        .expect("load salesforce gate");

    let default_included = sqlx::query_scalar::<_, bool>(
        "SELECT default_included FROM access_control_entities
         WHERE entity_type = 'plugin' AND entity_id = $1",
    )
    .bind(&plugin)
    .fetch_one(&*db.pool)
    .await
    .expect("catalog row exists");
    assert!(
        !default_included,
        "the gate owns the final word on defaults"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn load_salesforce_from_yaml_retracts_grants_the_file_no_longer_carries() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let first = unique("sf-first");
    let second = unique("sf-second");
    write_services_file(
        dir.path(),
        "access-control/salesforce.yaml",
        &salesforce_yaml("plugin", &first),
    );
    load_salesforce_from_yaml(&db.pool, dir.path())
        .await
        .expect("first load");

    write_services_file(
        dir.path(),
        "access-control/salesforce.yaml",
        &salesforce_yaml("plugin", &second),
    );
    load_salesforce_from_yaml(&db.pool, dir.path())
        .await
        .expect("second load");

    assert_eq!(linked_rule_entities(&db.pool).await, vec![second]);

    db.cleanup().await;
}

#[tokio::test]
async fn load_salesforce_from_yaml_deletes_the_pre_gate_role_grant() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    insert_acl_entity(&db.pool, "mcp_server", "salesforce", true).await;
    sqlx::query(
        "INSERT INTO access_control_rules (entity_type, entity_id, rule_type, rule_value, access)
         VALUES ('mcp_server', 'salesforce', 'role', 'user', 'allow')",
    )
    .execute(&*db.pool)
    .await
    .expect("seed the legacy grant");
    write_services_file(
        dir.path(),
        "access-control/salesforce.yaml",
        &salesforce_yaml("mcp_server", "salesforce"),
    );

    load_salesforce_from_yaml(&db.pool, dir.path())
        .await
        .expect("load salesforce gate");

    let legacy = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM access_control_rules
         WHERE entity_type = 'mcp_server' AND entity_id = 'salesforce'
           AND rule_type = 'role' AND rule_value = 'user'",
    )
    .fetch_one(&*db.pool)
    .await
    .expect("count legacy grant");
    assert_eq!(
        legacy, 0,
        "roles.yaml ingestion never deletes orphans, so the gate loader must"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn load_salesforce_from_yaml_rejects_an_unknown_entity_type() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    write_services_file(
        dir.path(),
        "access-control/salesforce.yaml",
        &salesforce_yaml("teapot", "x"),
    );

    let result = load_salesforce_from_yaml(&db.pool, dir.path()).await;

    assert!(result.is_err(), "an unknown entity kind must not be stored");

    db.cleanup().await;
}
