//! The audience grid narrows by query, focuses on one subject, drills into
//! one entity, and carries the ledger's reason into the inspector.

use axum::http::StatusCode;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

const MARKETPLACE: &str = "enterprise-demo";

async fn seed_developer_deny(db: &TempDb, reason: &str) {
    sqlx::query(
        "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
         VALUES ('marketplace', $1, true, 'dashboard')
         ON CONFLICT (entity_type, entity_id) DO NOTHING",
    )
    .bind(MARKETPLACE)
    .execute(&*db.pool)
    .await
    .expect("register the marketplace entity");
    sqlx::query(
        "INSERT INTO access_control_rules
             (id, entity_type, entity_id, rule_type, rule_value, access, justification, source)
         VALUES ($1, 'marketplace', $2, 'role', 'developer', 'deny', $3, 'dashboard')",
    )
    .bind(seed::unique("audience-deny"))
    .bind(MARKETPLACE)
    .bind(reason)
    .execute(&*db.pool)
    .await
    .expect("write the developer deny");
}

fn count(hay: &str, needle: &str) -> usize {
    hay.matches(needle).count()
}

#[tokio::test(flavor = "multi_thread")]
async fn audience_grid_groups_axes_and_marks_decisions() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let reason = seed::unique("developers audit the commons elsewhere");
    seed_developer_deny(&db, &reason).await;

    let (status, body) = app
        .call(Call::get(
            "/admin/access-control?tab=audience",
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "audience grid: {body}");
    assert!(
        body.contains("sp-ac-audience__band--role")
            && body.contains("data-entity-kind=\"marketplace\""),
        "columns are banded by subject kind and rows grouped by entity kind: {body}"
    );
    assert!(
        body.contains("is-allow") && body.contains("is-deny"),
        "cells carry the decision, not only the band: {body}"
    );
    assert!(
        body.contains("aria-label=\"How to read a cell\"") && body.contains("Denied"),
        "the legend names the decisions: {body}"
    );
    assert!(
        body.contains("name=\"subject_kind\"")
            && body.contains("name=\"decision\"")
            && body.contains("name=\"band\""),
        "the grid is narrowed by the same GET form as the Rules tab: {body}"
    );

    let (status, body) = app
        .call(Call::get(
            "/admin/access-control?tab=audience&subject=role%3Adeveloper",
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "subject focus: {body}");
    assert_eq!(
        count(&body, "class=\"sp-ac-audience__col "),
        1,
        "focus keeps exactly one column: {body}"
    );
    assert!(
        body.contains("is-focused"),
        "the focused column is marked: {body}"
    );
    assert!(
        body.contains("id=\"ac-aud-focus-title\"") && body.contains(&reason),
        "the inspector lists the entity with the ledger's reason: {body}"
    );

    let (status, body) = app
        .call(Call::get(
            "/admin/access-control?tab=audience&entity=marketplace%2Fenterprise-demo",
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "entity drill: {body}");
    assert_eq!(
        count(&body, "class=\"sp-ac-audience__row\""),
        1,
        "the drill keeps exactly one row: {body}"
    );
    assert!(
        body.contains("All subjects on screen") && body.contains(&reason),
        "the drill buckets every subject and carries the reason: {body}"
    );

    let (status, body) = app
        .call(Call::get(
            "/admin/access-control?tab=audience&subject=role%3Adeveloper&decision=deny",
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "decision filter: {body}");
    assert!(
        body.contains("is-deny") && !body.contains("class=\"sp-ac-audience__cell is-allow\""),
        "a decision filter drops rows the subject is allowed everywhere: {body}"
    );

    let (status, body) = app
        .call(Call::get(
            "/admin/access-control?tab=audience&entity_kind=hook&q=no-such-entity-anywhere",
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "empty narrowing: {body}");
    assert!(
        body.contains("Nothing matches these filters"),
        "an over-narrowed grid says so instead of blanking: {body}"
    );
    db.cleanup().await;
}
