//! `repositories::config::acl_yaml_loader` and `::acl_yaml_snapshot` — the
//! role-rule bootstrap and its inverse.

use systemprompt_security::authz::{Access, EntityKind, RuleType};
use systemprompt_web_admin::authz::department::department_rule_type;
use systemprompt_web_admin::repositories::config::acl_yaml_loader::load_from_yaml;
use systemprompt_web_admin::repositories::config::acl_yaml_snapshot::render_yaml_snapshot;
use systemprompt_web_admin::repositories::config::gateway_acl;

use crate::fixtures::{insert_acl_entity, unique, write_services_file};
use crate::tempdb::TempDb;

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
        department_rule_type(),
        "some-department",
        Access::Allow,
    )
    .await
    .expect("upsert department rule");

    let yaml = render_yaml_snapshot(&db.pool)
        .await
        .expect("render snapshot");

    assert!(
        !yaml.contains("some-department"),
        "roles.yaml carries role rules only; a department rule must be dropped"
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
async fn load_from_yaml_ingests_departments() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let name = unique("dept");
    write_services_file(
        dir.path(),
        "access-control/departments.yaml",
        &format!("departments:\n  - name: {name}\n    description: Test department\n"),
    );

    load_from_yaml(&db.pool, dir.path())
        .await
        .expect("first boot");
    load_from_yaml(&db.pool, dir.path())
        .await
        .expect("second boot");

    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM departments WHERE name = $1")
        .bind(&name)
        .fetch_one(&*db.pool)
        .await
        .expect("count departments");
    assert_eq!(
        count, 1,
        "the bootstrap upserts by name rather than appending"
    );

    db.cleanup().await;
}
