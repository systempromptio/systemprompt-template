//! `rules.yaml` against a live database: the three sync directions and the
//! state row every apply leaves. The boot contract itself (seed when empty,
//! otherwise compare) is pinned plane-independently in the unit workspace
//! (`sync_boot_contract`).
//!
//! Every test writes its own `access-control/rules.yaml` into a temp services
//! tree and runs against its own throwaway database, so the shipped file and
//! the shared tables are never touched.

use std::path::Path;

use sqlx::PgPool;
use systemprompt_security::authz::{DASHBOARD_SOURCE, RegisteredEntities};
use systemprompt_web_admin::repositories::access_control::declared::{
    DeclaredInputs, build_declared_set,
};
use systemprompt_web_admin::repositories::access_control::declared_load::load_declared_set;
use systemprompt_web_admin::repositories::access_control::drift::compute_drift;
use systemprompt_web_admin::repositories::access_control::export::{Owners, render_export};
use systemprompt_web_admin::repositories::access_control::rules::{
    list_band_rules, list_entity_defaults,
};
use systemprompt_web_admin::repositories::access_control::sync::{SyncMode, apply_sync};
use systemprompt_web_admin::repositories::config::rules_yaml_loader::parse_rules_doc;

use crate::fixtures::{insert_acl_entity, unique, write_services_file};
use crate::tempdb::TempDb;

const RULES: &str = "access-control/rules.yaml";

fn rules_yaml(server: &str, groups: &str) -> String {
    format!(
        "entities:\n  - entity: mcp_server/{server}\n    default: closed\n    why: pilot connector\n    allow:\n      role: [admin]\n      group: [{groups}]\n"
    )
}

async fn declared(
    dir: &Path,
) -> systemprompt_web_admin::repositories::access_control::declared::DeclaredSet {
    load_declared_set(dir, &[], &RegisteredEntities::default())
        .await
        .expect("declared set")
}

async fn rule_rows(pool: &PgPool, server: &str) -> Vec<(String, String, String, String)> {
    sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT rule_type, rule_value, access, source FROM access_control_rules
         WHERE entity_type = 'mcp_server' AND entity_id = $1
         ORDER BY rule_type, rule_value",
    )
    .bind(server)
    .fetch_all(pool)
    .await
    .expect("rule rows")
}

async fn insert_rule(pool: &PgPool, server: &str, rule_type: &str, value: &str, source: &str) {
    insert_acl_entity(pool, "mcp_server", server, false).await;
    sqlx::query(
        "INSERT INTO access_control_rules (id, entity_type, entity_id, rule_type, rule_value, access, justification, source)
         VALUES ($1, 'mcp_server', $2, $3, $4, 'allow', 'hand-written', $5)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(server)
    .bind(rule_type)
    .bind(value)
    .bind(source)
    .execute(pool)
    .await
    .expect("insert rule");
}

#[tokio::test]
async fn insert_only_adds_the_missing_and_leaves_everything_else() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let server = unique("srv");
    write_services_file(dir.path(), RULES, &rules_yaml(&server, "india-devs"));
    insert_rule(&db.pool, &server, "group", "uk", DASHBOARD_SOURCE).await;

    let set = declared(dir.path()).await;
    let outcome = apply_sync(&db.pool, &set, SyncMode::InsertOnly)
        .await
        .expect("insert only");
    assert_eq!(outcome.inserted, 2);
    assert_eq!(outcome.deleted, 0);

    let rows = rule_rows(&db.pool, &server).await;
    assert!(
        rows.iter()
            .any(|r| r.0 == "group" && r.1 == "uk" && r.3 == DASHBOARD_SOURCE)
    );
    assert!(
        rows.iter()
            .any(|r| r.0 == "group" && r.1 == "india-devs" && r.3 == "yaml")
    );
    assert!(rows.iter().any(|r| r.0 == "role" && r.1 == "admin"));

    let drift = compute_drift(
        &set,
        &list_band_rules(&*db.pool).await.expect("rules"),
        &list_entity_defaults(&*db.pool).await.expect("entities"),
    );
    assert_eq!(
        drift.counts().only_in_db_dashboard,
        1,
        "the console row is reported, not removed"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn overwrite_corrects_and_deletes_governed_rows_but_spares_user_and_undeclared() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let server = unique("srv");
    let other = unique("other");
    write_services_file(dir.path(), RULES, &rules_yaml(&server, "india-devs"));
    insert_rule(&db.pool, &server, "group", "uk", "yaml").await;
    insert_rule(&db.pool, &server, "user", "someone", DASHBOARD_SOURCE).await;
    insert_rule(&db.pool, &other, "group", "uk", DASHBOARD_SOURCE).await;
    sqlx::query(
        "UPDATE access_control_entities SET default_included = true
         WHERE entity_type = 'mcp_server' AND entity_id = $1",
    )
    .bind(&server)
    .execute(&*db.pool)
    .await
    .expect("open the entity");

    let set = declared(dir.path()).await;
    let outcome = apply_sync(&db.pool, &set, SyncMode::Overwrite)
        .await
        .expect("overwrite");
    assert_eq!(outcome.inserted, 2);
    assert_eq!(
        outcome.deleted, 1,
        "the governed row code no longer declares goes"
    );
    assert_eq!(outcome.entities_updated, 1, "the default is corrected");

    let rows = rule_rows(&db.pool, &server).await;
    assert!(!rows.iter().any(|r| r.0 == "group" && r.1 == "uk"));
    assert!(
        rows.iter().any(|r| r.0 == "user" && r.1 == "someone"),
        "user band untouched"
    );
    assert_eq!(
        rule_rows(&db.pool, &other).await.len(),
        1,
        "a console row on an undeclared entity is kept"
    );
    let closed: bool = sqlx::query_scalar(
        "SELECT default_included FROM access_control_entities
         WHERE entity_type = 'mcp_server' AND entity_id = $1",
    )
    .bind(&server)
    .fetch_one(&*db.pool)
    .await
    .expect("default");
    assert!(!closed);

    db.cleanup().await;
}

#[tokio::test]
async fn overwrite_retires_rows_code_wrote_on_an_entity_it_dropped_and_its_default() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let server = unique("srv");
    let dropped = unique("dropped");
    write_services_file(dir.path(), RULES, &rules_yaml(&server, "india-devs"));
    insert_rule(&db.pool, &dropped, "group", "uk", "yaml").await;
    insert_rule(&db.pool, &dropped, "role", "admin", "yaml").await;
    sqlx::query(
        "UPDATE access_control_entities SET source = $2
         WHERE entity_type = 'mcp_server' AND entity_id = $1",
    )
    .bind(&dropped)
    .bind(systemprompt_web_admin::repositories::access_control::sync::RULES_SOURCE)
    .execute(&*db.pool)
    .await
    .expect("stamp the entity as the file's");

    let set = declared(dir.path()).await;
    let outcome = apply_sync(&db.pool, &set, SyncMode::Overwrite)
        .await
        .expect("overwrite");
    assert_eq!(
        outcome.deleted, 2,
        "code removed the entity; code takes its rows away"
    );
    assert_eq!(outcome.entities_retired, 1, "and its default row with them");
    assert!(rule_rows(&db.pool, &dropped).await.is_empty());
    let defaults: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM access_control_entities
         WHERE entity_type = 'mcp_server' AND entity_id = $1",
    )
    .bind(&dropped)
    .fetch_one(&*db.pool)
    .await
    .expect("count");
    assert_eq!(defaults, 0);

    db.cleanup().await;
}

#[tokio::test]
async fn export_round_trips_through_the_loader() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let server = unique("srv");
    write_services_file(dir.path(), RULES, &rules_yaml(&server, "india-devs, uk"));
    let set = declared(dir.path()).await;
    apply_sync(&db.pool, &set, SyncMode::Overwrite)
        .await
        .expect("seed");

    let rules: Vec<_> = list_band_rules(&*db.pool)
        .await
        .expect("rules")
        .into_iter()
        .filter(|r| r.entity_id == server)
        .collect();
    let entities: Vec<_> = list_entity_defaults(&*db.pool)
        .await
        .expect("entities")
        .into_iter()
        .filter(|e| e.entity_id == server)
        .collect();
    let yaml = render_export(&rules, &entities, &Owners::default());
    let doc = parse_rules_doc(&yaml).expect("export parses as rules.yaml");
    let again = build_declared_set(
        &doc,
        &DeclaredInputs {
            gateway_routes: &[],
            marketplace_ids: &[],
            registered: &RegisteredEntities::default(),
        },
    )
    .expect("re-projects");
    assert_eq!(again.rules, set.rules);
    assert_eq!(again.entities, set.entities);

    db.cleanup().await;
}

#[tokio::test]
async fn sync_state_records_the_seed_and_every_apply() {
    use systemprompt_web_admin::repositories::sync::state::{
        Applied, AppliedFrom, BOOT_ACTOR, find_sync_state, record_applied, record_declared,
    };
    let Some(db) = TempDb::create().await else {
        return;
    };
    let plane = unique("plane");

    record_declared(&db.pool, &plane, "aaaa")
        .await
        .expect("declared");
    let row = find_sync_state(&db.pool, &plane)
        .await
        .expect("read")
        .expect("row exists");
    assert_eq!(row.declared_hash, "aaaa");
    assert!(
        row.applied_hash.is_none(),
        "a read never counts as an apply"
    );

    record_applied(
        &db.pool,
        &Applied {
            plane: &plane,
            declared_hash: "aaaa",
            mode: None,
            actor: BOOT_ACTOR,
            from: AppliedFrom::default(),
        },
    )
    .await
    .expect("seed");
    let row = find_sync_state(&db.pool, &plane)
        .await
        .expect("read")
        .expect("row");
    assert_eq!(row.applied_mode.as_deref(), Some("seed"));
    assert_eq!(row.applied_by.as_deref(), Some(BOOT_ACTOR));
    assert_eq!(row.applied_hash.as_deref(), Some("aaaa"));

    record_declared(&db.pool, &plane, "bbbb")
        .await
        .expect("declared moved");
    let row = find_sync_state(&db.pool, &plane)
        .await
        .expect("read")
        .expect("row");
    assert_eq!(row.declared_hash, "bbbb");
    assert_eq!(
        row.applied_hash.as_deref(),
        Some("aaaa"),
        "the apply is untouched by a read"
    );

    record_applied(
        &db.pool,
        &Applied {
            plane: &plane,
            declared_hash: "bbbb",
            mode: Some(SyncMode::Overwrite),
            actor: "ed",
            from: AppliedFrom {
                base_tree_hash: Some("tree".to_owned()),
                composed_hash: None,
            },
        },
    )
    .await
    .expect("apply");
    let row = find_sync_state(&db.pool, &plane)
        .await
        .expect("read")
        .expect("row");
    assert_eq!(row.applied_mode.as_deref(), Some("overwrite"));
    assert_eq!(row.applied_by.as_deref(), Some("ed"));
    assert_eq!(row.base_tree_hash.as_deref(), Some("tree"));
    assert!(row.applied_at.is_some());

    db.cleanup().await;
}
