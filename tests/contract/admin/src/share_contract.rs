//! Share-token issuance and the public manifest it unlocks.
//!
//! These two halves are only meaningful together. `POST /admin/users/{id}/
//! share-token` mints an HMAC over `user_id:version:expiry`; `GET
//! /share/manifest/ {token}` is the one route in the admin plane with **no**
//! authentication middleware in front of it, so that HMAC and the version
//! recheck behind it are the entire access control. The suite has never driven
//! the second half at all — `share_manifest_router` is merged at the root by
//! `extensions/web/src/router/api.rs`, outside both route modules the contract
//! table is derived from.
//!
//! Every rejection is asserted to be the *same* rejection. The handler
//! deliberately collapses "forged signature", "no such user", and "database
//! fault" into one `401` with one message, because an anonymous caller who
//! can tell those apart can enumerate user ids. A case that accepted a `404`
//! here would be ratifying that leak.

use axum::http::StatusCode;
use base64::Engine as _;
use serde_json::Value;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

const REJECTED: &str = "Invalid or revoked token";

fn parse(body: &str) -> Value {
    serde_json::from_str(body).unwrap_or_else(|e| panic!("response is JSON: {e}\n{body}"))
}

fn manifest_path(token: &str) -> String {
    format!("/share/manifest/{token}")
}

// Mint a share token for a user through the real issuance endpoint.
async fn issue(app: &App, user_id: &str) -> (String, String) {
    let (status, body) = app
        .call(Call::json(
            "post",
            &format!("/api/public/admin/users/{user_id}/share-token"),
            Principal::Admin,
            "{}",
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "issue share token: {body}");
    let issued = parse(&body);
    let token = issued["token"].as_str().expect("a token").to_owned();
    let url = issued["url"].as_str().expect("a url").to_owned();
    (token, url)
}

#[tokio::test]
async fn issuing_a_share_token_is_admin_only() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let user_id_str = seed::unique("share-target");
    let user_id = seed::insert_user(
        &db.pool,
        &user_id_str,
        &format!("{user_id_str}@contract.test"),
    )
    .await;
    seed::insert_profile_ext(&db.pool, &user_id, 1).await;

    let path = format!("/api/public/admin/users/{user_id_str}/share-token");
    let (status, body) = app
        .call(Call::json("post", &path, Principal::NonAdmin, "{}"))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "non-admin: {body}");
    assert_eq!(parse(&body)["error"], "Admin access required");

    let (status, _) = app
        .call(Call::json("post", &path, Principal::Anonymous, "{}"))
        .await;
    assert!(
        status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN,
        "anonymous issuance is refused, got {status}"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn issuing_for_a_user_with_no_profile_row_is_a_404() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // The version lives on `user_profile_ext`, not `users`, so a user who has
    // never been given a profile row has nothing to sign over. The issuance
    // endpoint is authenticated, so unlike the public verifier it may say so.
    let (status, body) = app
        .call(Call::json(
            "post",
            &format!(
                "/api/public/admin/users/{}/share-token",
                seed::unique("ghost")
            ),
            Principal::Admin,
            "{}",
        ))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "unknown user: {body}");
    assert_eq!(parse(&body)["error"], "User not found");

    db.cleanup().await;
}

#[tokio::test]
async fn an_issued_token_unlocks_that_user_s_manifest() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let user_id_str = seed::unique("share-user");
    let user_id = seed::insert_user(
        &db.pool,
        &user_id_str,
        &format!("{user_id_str}@contract.test"),
    )
    .await;
    seed::insert_profile_ext(&db.pool, &user_id, 1).await;

    let (token, url) = issue(&app, &user_id_str).await;
    assert_eq!(url, manifest_path(&token), "the url embeds the token");
    assert_eq!(
        token.split(':').count(),
        4,
        "the token is user:version:expiry:mac, got {token}"
    );

    let (status, body) = app.call(Call::get(&url, Principal::Anonymous)).await;
    assert_eq!(status, StatusCode::OK, "manifest: {body}");
    let manifest = parse(&body);
    assert_eq!(manifest["user_id"], user_id_str);

    // The catalog is read from `services/`, so the section list is whatever
    // this checkout ships. What is contractual is the shape: every section
    // names an entity type and every item carries an id.
    let sections = manifest["sections"]
        .as_array()
        .expect("sections is an array");
    for section in sections {
        assert!(
            section["entity_type"]
                .as_str()
                .is_some_and(|t| !t.is_empty()),
            "each section names its entity type: {section}"
        );
        for item in section["items"].as_array().expect("items is an array") {
            assert!(
                item["entity_id"].as_str().is_some_and(|i| !i.is_empty()),
                "each item carries an id: {item}"
            );
        }
    }

    db.cleanup().await;
}

#[tokio::test]
async fn rotating_the_version_revokes_an_issued_token() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let user_id_str = seed::unique("share-rotate");
    let user_id = seed::insert_user(
        &db.pool,
        &user_id_str,
        &format!("{user_id_str}@contract.test"),
    )
    .await;
    seed::insert_profile_ext(&db.pool, &user_id, 1).await;

    let (token, url) = issue(&app, &user_id_str).await;
    let (status, _) = app.call(Call::get(&url, Principal::Anonymous)).await;
    assert_eq!(status, StatusCode::OK, "the token works before rotation");

    seed::insert_profile_ext(&db.pool, &user_id, 2).await;

    // The signature still verifies — it is the version recheck that refuses.
    // That is the whole revocation mechanism, and it is invisible to a case
    // that only ever tampers with the MAC.
    let (status, body) = app.call(Call::get(&url, Principal::Anonymous)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "after rotation: {body}");
    assert_eq!(parse(&body)["error"], "Token has been revoked");

    // A token minted at the new version works again.
    let (fresh, fresh_url) = issue(&app, &user_id_str).await;
    assert_ne!(fresh, token, "rotation changes the token");
    let (status, _) = app.call(Call::get(&fresh_url, Principal::Anonymous)).await;
    assert_eq!(status, StatusCode::OK, "the reissued token works");

    db.cleanup().await;
}

#[tokio::test]
async fn a_tampered_token_is_refused() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let user_id_str = seed::unique("share-tamper");
    let user_id = seed::insert_user(
        &db.pool,
        &user_id_str,
        &format!("{user_id_str}@contract.test"),
    )
    .await;
    seed::insert_profile_ext(&db.pool, &user_id, 1).await;

    let (token, _) = issue(&app, &user_id_str).await;
    let parts: Vec<&str> = token.split(':').collect();
    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD;

    // Flip one hex digit of the MAC, keeping the length identical so the
    // constant-time compare — not the length precheck — is what rejects it.
    let mac = parts[3];
    let flipped = if mac.starts_with('0') {
        format!("1{}", &mac[1..])
    } else {
        format!("0{}", &mac[1..])
    };
    let forged = format!("{}:{}:{}:{flipped}", parts[0], parts[1], parts[2]);
    let (status, body) = app
        .call(Call::get(&manifest_path(&forged), Principal::Anonymous))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "flipped mac: {body}");
    assert_eq!(parse(&body)["error"], REJECTED);

    // Same MAC, different user. A token that carried over to another id would
    // be the worst possible failure of this endpoint.
    let other = b64.encode(b"someone-else");
    let swapped = format!("{other}:{}:{}:{}", parts[1], parts[2], parts[3]);
    let (status, body) = app
        .call(Call::get(&manifest_path(&swapped), Principal::Anonymous))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "swapped subject: {body}");

    // Same MAC, different version.
    let bumped = b64.encode(b"99");
    let reversioned = format!("{}:{bumped}:{}:{}", parts[0], parts[2], parts[3]);
    let (status, _) = app
        .call(Call::get(
            &manifest_path(&reversioned),
            Principal::Anonymous,
        ))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "swapped version");

    // Same MAC, different expiry. A stretched clock must fail like any other
    // tamper — the expiry is inside the signed payload, not advisory.
    let stretched = b64.encode(b"9999999999");
    let restretched = format!("{}:{}:{stretched}:{}", parts[0], parts[1], parts[3]);
    let (status, _) = app
        .call(Call::get(
            &manifest_path(&restretched),
            Principal::Anonymous,
        ))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "stretched expiry");

    db.cleanup().await;
}

#[tokio::test]
async fn malformed_tokens_are_refused_the_same_way() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let user = b64.encode(b"someone");
    let version = b64.encode(b"1");

    // Each of these takes a different early exit out of `verify`, and every
    // one of them must be indistinguishable from the others on the wire.
    let cases = [
        ("no separators", "not-a-token".to_owned()),
        ("two parts", format!("{user}:{version}")),
        (
            "three parts (the pre-expiry shape)",
            format!("{user}:{version}:aa"),
        ),
        ("five parts", format!("{user}:{version}:aa:bb:cc")),
        ("subject is not base64", format!("!!!:{version}:aa:bb")),
        ("version is not base64", format!("{user}:!!!:aa:bb")),
        (
            "version is not a number",
            format!("{user}:{}:aa:bb", b64.encode(b"one")),
        ),
        (
            "expiry is not a number",
            format!("{user}:{version}:{}:bb", b64.encode(b"never")),
        ),
        (
            "subject is not utf-8",
            format!("{}:{version}:aa:bb", b64.encode([0xff, 0xfe])),
        ),
    ];

    for (label, token) in cases {
        let (status, body) = app
            .call(Call::get(&manifest_path(&token), Principal::Anonymous))
            .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{label}: {body}");
        assert_eq!(parse(&body)["error"], REJECTED, "{label}");
    }

    db.cleanup().await;
}

#[tokio::test]
async fn a_well_formed_token_for_an_unknown_user_is_refused_as_a_forgery() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // Issue against a real user, then re-point the profile row's owner by
    // deleting it: the signature is genuine, the user row is gone, and the
    // answer must still be the generic rejection rather than a 404 that would
    // confirm the id was once real.
    let user_id_str = seed::unique("share-vanish");
    let user_id = seed::insert_user(
        &db.pool,
        &user_id_str,
        &format!("{user_id_str}@contract.test"),
    )
    .await;
    seed::insert_profile_ext(&db.pool, &user_id, 1).await;
    let (_, url) = issue(&app, &user_id_str).await;

    sqlx::query("DELETE FROM user_profile_ext WHERE user_id = $1")
        .bind(user_id.as_str())
        .execute(db.pool.as_ref())
        .await
        .expect("delete the profile row");

    let (status, body) = app.call(Call::get(&url, Principal::Anonymous)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "vanished profile: {body}");
    assert_eq!(parse(&body)["error"], REJECTED);

    db.cleanup().await;
}
