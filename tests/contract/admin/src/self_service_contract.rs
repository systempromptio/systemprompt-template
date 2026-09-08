//! `PUT /user/settings` and `DELETE /user/account` — the two writes scoped to
//! the caller rather than to a role.
//!
//! Every other write under `/api/public/admin` is behind an admin role, and
//! these two deliberately are not: a developer with no console access still
//! owns their own display name. That makes the interesting question the
//! opposite of the usual one. It is not "who is refused" — only anonymous is —
//! but "can a caller reach somebody else's row", and the answer has to be no
//! by construction: there is no user id in the path and none in the body, so
//! the session is the only thing that names the target.

use axum::http::StatusCode;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

const SETTINGS: &str = "/api/public/admin/user/settings";
const ACCOUNT: &str = "/api/public/admin/user/account";
const BODY: &str =
    r#"{"display_name":"Contract Tester","avatar_url":null,"timezone":"Europe/Madrid"}"#;

// Why: a bodyless DELETE, which `Call` has no constructor for — `get` and
// `json` cover every other call in the suite. This is the shape the route
// prober sends, and the shape the route must refuse.
const fn bare_delete(principal: Principal) -> Call<'static> {
    Call {
        method: "delete",
        path: ACCOUNT,
        principal,
        content_type: None,
        body: None,
    }
}

#[tokio::test]
async fn every_signed_in_principal_saves_their_own_settings() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    for principal in Principal::ALL_DASHBOARD
        .into_iter()
        .filter(|p| *p != Principal::Anonymous)
    {
        let (status, body) = app.call(Call::json("put", SETTINGS, principal, BODY)).await;
        assert_eq!(status, StatusCode::OK, "{principal:?} body: {body}");
        let json: serde_json::Value = serde_json::from_str(&body).expect("json");
        assert_eq!(json["timezone"], "Europe/Madrid", "{principal:?}");
        assert_eq!(json["display_name"], "Contract Tester", "{principal:?}");
    }
}

#[tokio::test]
async fn the_saved_row_belongs_to_the_caller_and_nobody_else() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, body) = app
        .call(Call::json("put", SETTINGS, Principal::NonAdmin, BODY))
        .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let json: serde_json::Value = serde_json::from_str(&body).expect("json");
    let owner = json["user_id"].as_str().expect("user_id");

    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_settings")
        .fetch_one(&*db.pool)
        .await
        .expect("count settings");
    assert_eq!(rows, 1, "one save wrote one row");

    let stored: String = sqlx::query_scalar("SELECT user_id FROM user_settings")
        .fetch_one(&*db.pool)
        .await
        .expect("read settings");
    assert_eq!(stored, owner, "the row is the caller's own");
}

#[tokio::test]
async fn anonymous_is_refused_both_writes() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, _) = app
        .call(Call::json("put", SETTINGS, Principal::Anonymous, BODY))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = app.call(bare_delete(Principal::Anonymous)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// Why: the value is used to render every timestamp the console shows this
// person, so a string that cannot be a zone name is refused rather than stored
// and misread for as long as the account lives.
#[tokio::test]
async fn a_timezone_that_cannot_be_a_zone_name_is_refused() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    for bad in [
        r#"{"timezone":""}"#,
        r#"{"timezone":"Europe/Madrid; DROP TABLE users"}"#,
        r#"{"timezone":"<script>alert(1)</script>"}"#,
    ] {
        let (status, body) = app
            .call(Call::json("put", SETTINGS, Principal::NonAdmin, bad))
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "accepted {bad}: {body}");
    }
}

// Why: irreversible, so what it removes and what it leaves are both part of
// the contract. The account and its settings go; the gateway's record of what
// the account did stays, because `ai_requests` holds no foreign key to `users`
// and an audit trail a person can erase by closing their account is not one.
#[tokio::test]
async fn deleting_an_account_removes_it_and_leaves_the_audit_trail() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, body) = app
        .call(Call::json("put", SETTINGS, Principal::NonAdmin, BODY))
        .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let json: serde_json::Value = serde_json::from_str(&body).expect("json");
    let owner = json["user_id"].as_str().expect("user_id").to_owned();

    let request_id = crate::seed::unique("self-service-audit");
    let owner_id = systemprompt::identifiers::UserId::new(owner.clone());
    crate::seed::insert_request(
        &db.pool,
        &crate::seed::RequestSpec {
            id: request_id.clone(),
            user_id: &owner_id,
            session_id: None,
            trace_id: None,
            context_id: None,
            status: "completed",
        },
    )
    .await;

    let email: String = sqlx::query_scalar("SELECT email FROM users WHERE id = $1")
        .bind(&owner)
        .fetch_one(&*db.pool)
        .await
        .expect("read email");

    // The guard, which is also what makes this route safe to leave mounted: a
    // DELETE naming no account deletes nothing, so the baseline prober's
    // bodyless call cannot close an account as a side effect of being recorded.
    let (status, _) = app.call(bare_delete(Principal::NonAdmin)).await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "a bare DELETE must not delete"
    );

    let wrong = r#"{"confirm_email":"someone-else@contract.test"}"#;
    let (status, _) = app
        .call(Call::json("delete", ACCOUNT, Principal::NonAdmin, wrong))
        .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "another account's name must not delete this one"
    );

    let confirmed = format!(r#"{{"confirm_email":"{email}"}}"#);
    let (status, body) = app
        .call(Call::json(
            "delete",
            ACCOUNT,
            Principal::NonAdmin,
            &confirmed,
        ))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "body: {body}");

    let users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = $1")
        .bind(&owner)
        .fetch_one(&*db.pool)
        .await
        .expect("count users");
    assert_eq!(users, 0, "the account is gone");

    let settings: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_settings WHERE user_id = $1")
        .bind(&owner)
        .fetch_one(&*db.pool)
        .await
        .expect("count settings");
    assert_eq!(settings, 0, "the settings row went with it, not orphaned");
    let audit_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ai_requests WHERE id = $1")
        .bind(request_id)
        .fetch_one(&*db.pool)
        .await
        .expect("audit row survives");
    assert_eq!(
        audit_rows, 1,
        "closing an account preserves its audit trail"
    );
}
