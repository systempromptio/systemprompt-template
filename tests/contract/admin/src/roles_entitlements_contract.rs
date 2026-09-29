//! The role ledger distinguishes role-band grants from closed defaults and
//! preserves both allow and deny rows rather than reporting only the reach.

use axum::http::StatusCode;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

#[tokio::test(flavor = "multi_thread")]
async fn role_entitlements_render_mixed_allow_and_deny_with_entity_defaults() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let allow = seed::unique("role-allow-skill");
    let deny = seed::unique("role-deny-skill");
    for (entity, default_included) in [(&allow, false), (&deny, true)] {
        sqlx::query(
            "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
             VALUES ('skill', $1, $2, 'dashboard')",
        )
        .bind(entity)
        .bind(default_included)
        .execute(&*db.pool)
        .await
        .expect("register entity");
    }
    for (entity, access) in [(&allow, "allow"), (&deny, "deny")] {
        sqlx::query(
            "INSERT INTO access_control_rules
                (id, entity_type, entity_id, rule_type, rule_value, access, justification, source)
             VALUES ($1, 'skill', $2, 'role', 'project_manager', $3,
                     'role entitlement fixture', 'dashboard')",
        )
        .bind(seed::unique("role-rule"))
        .bind(entity)
        .bind(access)
        .execute(&*db.pool)
        .await
        .expect("write role rule");
    }

    let (status, body) = app
        .call(Call::get("/admin/roles?tab=entitlements", Principal::Admin))
        .await;
    assert_eq!(status, StatusCode::OK, "role ledger: {body}");
    let role_start = body
        .find("aria-label=\"Project manager entitlements\"")
        .expect("project manager section");
    let role_body = &body[role_start..];
    let role_end = role_body.find("</section>").expect("role section ends");
    let role_body = &role_body[..role_end];
    assert!(role_body.contains(&allow));
    assert!(role_body.contains(&deny));
    assert!(
        role_body.contains("mixed"),
        "the fixture's allow and deny stay distinct"
    );
    assert!(role_body.contains("Closed by default"));
    assert!(role_body.contains("Open by default"));
    db.cleanup().await;
}
