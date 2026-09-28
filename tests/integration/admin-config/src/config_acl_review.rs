//! The per-entity access review against a live database: an apply scoped to
//! one entity moves that entity alone, and the review classifies what is
//! left — a declared entity never applied as *new in code*, a live entity
//! missing a declared rule as *removed in console*.

use std::path::Path;

use sqlx::PgPool;
use systemprompt_security::authz::RegisteredEntities;
use systemprompt_web_admin::repositories::access_control::declared::DeclaredSet;
use systemprompt_web_admin::repositories::access_control::declared_load::load_declared_set;
use systemprompt_web_admin::repositories::access_control::drift::compute_drift;
use systemprompt_web_admin::repositories::access_control::review::{
    EntityReview, ReviewKind, review_entities,
};
use systemprompt_web_admin::repositories::access_control::rules::{
    list_band_rules, list_entity_defaults,
};
use systemprompt_web_admin::repositories::access_control::sync::{
    EntityScope, SyncMode, apply_sync_scoped,
};

use crate::fixtures::{unique, write_services_file};
use crate::tempdb::TempDb;

const RULES: &str = "access-control/rules.yaml";

fn two_servers(a: &str, b: &str) -> String {
    format!(
        "entities:\n  - entity: mcp_server/{a}\n    default: closed\n    why: pilot\n    allow:\n      role: [admin]\n  - entity: mcp_server/{b}\n    default: closed\n    why: pilot\n    allow:\n      role: [admin]\n"
    )
}

async fn declared(dir: &Path) -> DeclaredSet {
    load_declared_set(dir, &[], &RegisteredEntities::default())
        .await
        .expect("declared set")
}

async fn review(pool: &PgPool, set: &DeclaredSet) -> Vec<EntityReview> {
    let drift = compute_drift(
        set,
        &list_band_rules(pool).await.expect("rules"),
        &list_entity_defaults(pool).await.expect("entities"),
    );
    review_entities(&drift)
}

fn kind_of(reviews: &[EntityReview], server: &str) -> Option<ReviewKind> {
    reviews
        .iter()
        .find(|r| r.key == format!("mcp_server/{server}"))
        .map(|r| r.kind)
}

async fn rule_count(pool: &PgPool, server: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM access_control_rules
         WHERE entity_type = 'mcp_server' AND entity_id = $1",
    )
    .bind(server)
    .fetch_one(pool)
    .await
    .expect("count")
}

#[tokio::test]
async fn a_scoped_apply_moves_one_entity_and_the_review_names_the_rest() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let applied = unique("srv");
    let pending = unique("srv");
    write_services_file(dir.path(), RULES, &two_servers(&applied, &pending));
    let set = declared(dir.path()).await;

    let before = review(&db.pool, &set).await;
    assert_eq!(kind_of(&before, &applied), Some(ReviewKind::NewInCode));
    assert!(
        before
            .iter()
            .filter(|r| r.key.ends_with(&applied))
            .all(|r| r.not_live),
        "never applied means not live"
    );

    let keys = [format!("mcp_server/{applied}")];
    let outcome = apply_sync_scoped(
        &db.pool,
        &set,
        SyncMode::Overwrite,
        EntityScope::Entities(&keys),
    )
    .await
    .expect("scoped apply");
    assert_eq!(outcome.inserted, 1);
    assert_eq!(outcome.entities_inserted, 1);
    assert_eq!(rule_count(&db.pool, &applied).await, 1);
    assert_eq!(
        rule_count(&db.pool, &pending).await,
        0,
        "the other entity is not touched"
    );

    let after = review(&db.pool, &set).await;
    assert_eq!(
        kind_of(&after, &applied),
        None,
        "the applied entity is settled"
    );
    assert_eq!(kind_of(&after, &pending), Some(ReviewKind::NewInCode));

    sqlx::query(
        "DELETE FROM access_control_rules
         WHERE entity_type = 'mcp_server' AND entity_id = $1",
    )
    .bind(&applied)
    .execute(&*db.pool)
    .await
    .expect("remove in console");
    let removed = review(&db.pool, &set).await;
    assert_eq!(
        kind_of(&removed, &applied),
        Some(ReviewKind::RemovedInConsole),
        "a live entity missing its declared rule reads as a console removal"
    );

    db.cleanup().await;
}
