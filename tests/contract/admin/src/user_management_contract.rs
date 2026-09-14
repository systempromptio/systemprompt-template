//! HTTP-level authorization contract for user provisioning: the
//! operator-created door and the boundary the admin role draws around it.
//!
//! These are the rules that decide who may mint an account and with which
//! roles, so they are asserted over the real router with real tokens rather
//! than at the repository layer, where the middleware and handler guards would
//! not run at all.

use axum::http::StatusCode;
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt_security::{AdminTokenParams, JwtService};

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

// A second admin, seeded plainly. The `admin` role alone is system-wide on
// this instance, so nothing distinguishes them from the fixture admin but the
// row — the cases below assert that the boundary is the role itself.
async fn seed_second_admin(pool: &PgPool) -> String {
    let user_id = UserId::new(format!("admin2-{}", uuid::Uuid::new_v4().simple()));
    let email = format!("{}@contract.test", user_id.as_str());
    sqlx::query(
        "INSERT INTO users (id, name, email, roles, email_verified)
         VALUES ($1, $1, $2, $3, true)",
    )
    .bind(user_id.as_str())
    .bind(&email)
    .bind(vec!["admin".to_owned(), "user".to_owned()])
    .execute(pool)
    .await
    .expect("seed second admin");
    let session_id = SessionId::new(uuid::Uuid::new_v4().to_string());
    JwtService::generate_admin_token(&AdminTokenParams {
        user_id: &user_id,
        session_id: &session_id,
        email: &email,
        issuer: &globals::jwt_issuer(),
        duration: chrono::Duration::hours(1),
        client_id: None,
    })
    .expect("mint a session token")
    .as_str()
    .to_owned()
}

// Why: this rule used to be asserted against `POST /invites`. Invites are gone
// — SSO is the only door and an AD group is the invitation — but the escalation
// rule they carried still governs the surviving operator-created door, so it is
// re-asserted there rather than deleted with them. What must hold is that the
// *admin role* is the boundary — a non-admin cannot mint an admin.
#[tokio::test]
async fn an_admin_may_create_a_user_with_elevated_roles() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let second_admin = seed_second_admin(&db.pool).await;

    let (status, body) = app
        .call_with_bearer(
            Call::json(
                "post",
                "/api/public/admin/users",
                Principal::Anonymous,
                r#"{"user_id":"acme-newcomer","display_name":"Newcomer","email":"newcomer@acme.test","roles":["admin"]}"#,
            ),
            &second_admin,
        )
        .await;

    assert_eq!(
        status,
        StatusCode::CREATED,
        "every admin is system-wide here, so granting the admin role is an \
         ordinary admin act: {body}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_non_admin_may_not_create_a_user_at_all() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // Why: the admin role is the only boundary, so it has to hold completely.
    let (status, body) = app
        .call(Call::json(
            "post",
            "/api/public/admin/users",
            Principal::NonAdmin,
            r#"{"user_id":"sneaky","display_name":"Sneaky","email":"sneaky@acme.test","roles":["admin"]}"#,
        ))
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "body: {body}");
    db.cleanup().await;
}

#[tokio::test]
async fn creating_a_user_hands_back_no_link_and_says_so() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, body) = app
        .call(Call::json(
            "post",
            "/api/public/admin/users",
            Principal::Admin,
            r#"{"user_id":"claimed-newcomer","display_name":"Newcomer","email":"newcomer@claimed.test","roles":["user"]}"#,
        ))
        .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let created: serde_json::Value = serde_json::from_str(&body).expect("created json");
    assert!(
        created.get("invite_path").is_none(),
        "there is no link to hand out — SSO is the only door: {body}"
    );
    assert!(
        created["invite_note"]
            .as_str()
            .is_some_and(|n| n.contains("SSO")),
        "the operator is told how the person signs in instead: {body}"
    );

    // Why: placement is the directory's to assign. An operator-created row
    // joins no group here and is placed at the person's first SSO sign-in,
    // until when the view derives them into `unassigned`.
    let groups: Vec<String> =
        sqlx::query_scalar("SELECT group_id FROM user_groups WHERE user_id = 'claimed-newcomer'")
            .fetch_all(&*db.pool)
            .await
            .expect("read the derived membership");
    assert_eq!(groups, vec!["unassigned".to_owned()]);
    db.cleanup().await;
}
