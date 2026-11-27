//! The console has no public signup: an anonymous registration request is not
//! served and creates no account, whatever role it asks for.

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::{globals, principal};
use template_test_common::TempDb;

#[tokio::test(flavor = "multi_thread")]
async fn public_registration_creates_no_account() {
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
    assert!(
        !status.is_success() && !status.is_server_error(),
        "an anonymous registration is refused, not served: {} {response}",
        status.as_u16()
    );
    let accounts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(db.pool.as_ref())
        .await
        .expect("count accounts");
    assert_eq!(accounts, 0, "no account was created");
    db.cleanup().await;
}
