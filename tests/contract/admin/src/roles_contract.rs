//! `GET|PUT /users/{id}/roles` — the rule the role editor applies, asserted
//! over the real router.
//!
//! Role editing left `PUT /users/{id}` because it is the one field with a
//! rule attached, and the rule is about who the caller is. That makes it a
//! router-and-token question, not a repository one: the tier the route sits
//! on and the checks inside the handler both have to hold.

use axum::http::StatusCode;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

async fn seed_user(pool: &PgPool, id: &str, roles: &[&str]) -> String {
    let user_id = UserId::new(format!("{id}-{}", uuid::Uuid::new_v4().simple()));
    sqlx::query(
        "INSERT INTO users (id, name, email, roles, email_verified)
         VALUES ($1, $1, $2, $3, true)",
    )
    .bind(user_id.as_str())
    .bind(format!("{}@contract.test", user_id.as_str()))
    .bind(roles.iter().map(|r| (*r).to_owned()).collect::<Vec<_>>())
    .execute(pool)
    .await
    .expect("seed user");
    user_id.as_str().to_owned()
}

#[tokio::test]
async fn an_admin_reads_the_two_halves_of_a_role_set() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let target = seed_user(&db.pool, "reader", &["user"]).await;

    let (status, body) = app
        .call(Call::get(
            &format!("/api/public/admin/users/{target}/roles"),
            Principal::Admin,
        ))
        .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    let json: serde_json::Value = serde_json::from_str(&body).expect("json");
    assert_eq!(json["roles"], serde_json::json!(["user"]));
    assert_eq!(
        json["manual_roles"],
        serde_json::json!([]),
        "nothing was granted by hand"
    );
    assert_eq!(
        json["directory_roles"],
        serde_json::json!([]),
        "local roles are not inferred to belong to a directory"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn an_admin_may_grant_an_ordinary_role() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let target = seed_user(&db.pool, "grantee", &["user"]).await;

    let (status, body) = app
        .call(Call::json(
            "put",
            &format!("/api/public/admin/users/{target}/roles"),
            Principal::Admin,
            r#"{"roles":["user","developer"]}"#,
        ))
        .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    let json: serde_json::Value = serde_json::from_str(&body).expect("json");
    assert_eq!(json["roles"], serde_json::json!(["developer", "user"]));
    assert_eq!(
        json["manual_roles"],
        serde_json::json!(["developer", "user"]),
        "local grants are recorded as manual roles"
    );
    db.cleanup().await;
}

// Why: the template admin role retains its existing account-management
// authority.
#[tokio::test]
async fn an_admin_retains_authority_to_grant_platform_admin() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let target = seed_user(&db.pool, "escalating", &["user"]).await;

    let (status, body) = app
        .call(Call::json(
            "put",
            &format!("/api/public/admin/users/{target}/roles"),
            Principal::Admin,
            r#"{"roles":["user","platform_admin"]}"#,
        ))
        .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    let roles: Vec<String> = sqlx::query_scalar("SELECT roles FROM users WHERE id = $1")
        .bind(&target)
        .fetch_one(&*db.pool)
        .await
        .expect("read back");
    assert_eq!(
        roles,
        vec!["platform_admin".to_owned(), "user".to_owned()],
        "the grant persisted"
    );
    db.cleanup().await;
}

// Why: configured entitlements use free-text roles beyond the built-in console
// tiers.
#[tokio::test]
async fn a_custom_role_is_persisted() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let target = seed_user(&db.pool, "typo", &["user"]).await;

    let (status, body) = app
        .call(Call::json(
            "put",
            &format!("/api/public/admin/users/{target}/roles"),
            Principal::Admin,
            r#"{"roles":["user","supervisor"]}"#,
        ))
        .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(
        body.contains("supervisor"),
        "the saved account retains the custom role: {body}"
    );
    db.cleanup().await;
}

// Why: reading roles is a console act and writing them is not. A project
// manager runs a project, not the role model.
#[tokio::test]
async fn a_project_manager_may_read_roles_but_not_set_them() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let target = seed_user(&db.pool, "watched", &["user"]).await;
    let path = format!("/api/public/admin/users/{target}/roles");

    let (read, _) = app.call(Call::get(&path, Principal::ProjectManager)).await;
    let (write, body) = app
        .call(Call::json(
            "put",
            &path,
            Principal::ProjectManager,
            r#"{"roles":["user","admin"]}"#,
        ))
        .await;

    assert_eq!(read, StatusCode::OK);
    assert_eq!(write, StatusCode::FORBIDDEN, "body: {body}");
    db.cleanup().await;
}

#[tokio::test]
async fn an_anonymous_caller_reaches_neither_verb() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let path = "/api/public/admin/users/anyone/roles";

    let (read, _) = app.call(Call::get(path, Principal::Anonymous)).await;
    let (write, _) = app
        .call(Call::json(
            "put",
            path,
            Principal::Anonymous,
            r#"{"roles":["admin"]}"#,
        ))
        .await;

    assert_eq!(read, StatusCode::UNAUTHORIZED);
    assert_eq!(write, StatusCode::UNAUTHORIZED);
    db.cleanup().await;
}
