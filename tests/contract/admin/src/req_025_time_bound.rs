//! REQ-025 Time-Bound External Access — "External/contractor access can be
//! scoped and configured to expire automatically at a defined time without
//! manual revocation."
//!
//! The share token is the platform's external-facing credential, and its
//! expiry is inside the signed token. The issuance endpoint only mints
//! thirty-day tokens, so this module signs with the same member signer the
//! handler verifies with and a short lifetime: a token inside its window is
//! accepted, which pins the signer to the real verifier, and the identical
//! token past its window is refused with the generic rejection, with no
//! revocation step in between.

use std::time::Duration;

use axum::http::StatusCode;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::{globals, principal, seed};
use template_test_common::TempDb;

fn sign(user_id: &str, version: i32, ttl: Duration) -> String {
    template_test_common::env::share_tokens()
        .sign(
            "share.manifest",
            format!("{user_id}:{version}").as_bytes(),
            ttl,
        )
        .expect("the fixture signer signs")
}

#[tokio::test]
async fn a_genuine_token_expires_on_its_own_without_revocation() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let user_id_str = seed::unique("share-expiry");
    let user_id = seed::insert_user(
        &db.pool,
        &user_id_str,
        &format!("{user_id_str}@contract.test"),
    )
    .await;
    seed::insert_profile_ext(&db.pool, &user_id, 1).await;

    // Within the window: proves this signer matches the handler's verifier,
    // so the expired case below fails on the clock, not on a MAC mismatch.
    let live = sign(&user_id_str, 1, Duration::from_secs(3600));
    let (status, body) = app
        .call(Call::get(
            &format!("/share/manifest/{live}"),
            Principal::Anonymous,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "a token inside its window: {body}");

    // Same user, same version, same signer — only the lifetime has run out.
    let expired = sign(&user_id_str, 1, Duration::from_secs(1));
    tokio::time::sleep(Duration::from_millis(2100)).await;
    let (status, body) = app
        .call(Call::get(
            &format!("/share/manifest/{expired}"),
            Principal::Anonymous,
        ))
        .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "an expired token is refused with nobody having revoked anything: {body}"
    );
    assert!(
        body.contains("Invalid or revoked token"),
        "expiry answers with the same generic rejection as a forgery: {body}"
    );

    db.cleanup().await;
}
