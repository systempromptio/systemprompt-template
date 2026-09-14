//! `GET /api/public/bridge/whoami`, driven end-to-end.
//!
//! The endpoint is merged into `/api/public` beside the admin API rather than
//! mounted inside either route module, so the derived table in
//! [`crate::route_source`] cannot see it and would never drive it. What that
//! would leave unexercised is the reason it exists: it authenticates from a
//! bearer token instead of the admin session, and it answers a *union* of five
//! tables, so a caller with no SSO mapping must still get
//! a body rather than the 500 an inner join would produce.
//!
//! Both non-admin and admin are driven, because the endpoint deliberately does
//! not gate on role — it describes whoever is asking, and a plain user asking
//! about themselves is the ordinary case on a desktop.

use axum::http::StatusCode;
use serde_json::Value;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

const PATH: &str = "/api/public/bridge/whoami";

fn parse(body: &str) -> Value {
    serde_json::from_str(body).unwrap_or_else(|e| panic!("response is JSON: {e}\n{body}"))
}

#[tokio::test]
async fn an_unauthenticated_caller_is_refused() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, _) = app.call(Call::get(PATH, Principal::Anonymous)).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "no token means no identity to describe"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_garbage_bearer_token_is_refused_rather_than_crashing() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, body) = app
        .call_with_bearer(Call::get(PATH, Principal::Anonymous), "not-a-jwt")
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    db.cleanup().await;
}

#[tokio::test]
async fn a_plain_user_with_no_sso_mapping_still_gets_a_complete_body() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, body) = app.call(Call::get(PATH, Principal::NonAdmin)).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let v = parse(&body);
    assert!(v["user_id"].is_string(), "identity is named: {body}");
    assert!(v["email"].is_string(), "{body}");
    assert_eq!(v["is_admin"], Value::Bool(false), "{body}");
    assert_eq!(
        v["groups"],
        serde_json::json!(["contract-group"]),
        "the group the directory placed the caller in is on the card: {body}"
    );
    assert!(
        v.get("project").is_none(),
        "the single-project column is gone from the card: {body}"
    );
    assert!(
        v.get("organization_id").is_none() && v.get("is_platform_admin").is_none(),
        "the organization model is gone from the card: {body}"
    );
    assert!(
        v.get("provider").is_none(),
        "a local account claims no identity provider: {body}"
    );
    assert!(
        v.get("directory_groups").is_none(),
        "an empty group list is omitted rather than rendered as a blank row: {body}"
    );
    assert!(
        v["token_expires_unix"].is_i64() && v["token_issuer"].is_string(),
        "the presented token describes itself: {body}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn an_admin_is_reported_as_one() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, body) = app.call(Call::get(PATH, Principal::Admin)).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let v = parse(&body);
    assert_eq!(v["is_admin"], Value::Bool(true), "{body}");
    assert!(
        v["project"].is_null(),
        "an operator admin who never came through AD FS has no project: {body}"
    );
    let roles: Vec<String> = serde_json::from_value(v["roles"].clone()).expect("roles is an array");
    assert!(roles.contains(&"admin".to_owned()), "{body}");
    db.cleanup().await;
}
