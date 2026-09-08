//! Public signup cannot choose privileged roles.

use axum::http::StatusCode;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

#[tokio::test(flavor = "multi_thread")]
async fn public_registration_never_grants_admin() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let email = format!("registration-{}@contract.test", uuid::Uuid::new_v4());
    let body =
        serde_json::json!({"name": "Public user", "email": email, "role": "admin"}).to_string();
    let (status, response) = app
        .call(Call::json(
            "post",
            "/admin/api/register",
            Principal::Anonymous,
            &body,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "{response}");
    let roles: Vec<String> = sqlx::query_scalar("SELECT roles FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(db.pool.as_ref())
        .await
        .expect("registered user");
    assert_eq!(roles, vec!["user"]);
    db.cleanup().await;
}
