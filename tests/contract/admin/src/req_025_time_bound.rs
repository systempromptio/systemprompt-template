//! REQ-025 Time-Bound External Access — "External/contractor access can be
//! scoped and configured to expire automatically at a defined time without
//! manual revocation."
//!
//! The share token is the platform's external-facing credential, and its
//! expiry instant is inside the signed payload. The existing share contract
//! proves tampering with the expiry fails the MAC; what it cannot prove is
//! that a *genuine* token dies on its own once its window passes — the
//! issuance endpoint only mints thirty-day tokens. So this module re-signs
//! with the same fixture seed the handler reads (HMAC-SHA256 over
//! `user_id:version:expiry`), first confirming a future-dated forgery is
//! accepted — which pins the re-implementation to the real verifier — and
//! then that the identical token with a past expiry is refused with the
//! generic rejection, with no revocation step in between.

use axum::http::StatusCode;
use base64::Engine as _;
use sha2::{Digest, Sha256};

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

// Byte-for-byte the token construction in
// `extensions/web/admin/src/handlers/share.rs::sign`, driven by the fixture
// seed `globals::init` installs. If the handler's algorithm drifts, the
// acceptance test below fails first.
fn sign(secret: &[u8], user_id: &str, version: i32, expires_unix: i64) -> String {
    let payload = format!("{user_id}:{version}:{expires_unix}");
    let mut padded = [0u8; 64];
    padded[..secret.len()].copy_from_slice(secret);
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= padded[i];
        opad[i] ^= padded[i];
    }
    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(payload.as_bytes());
    let inner_digest = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_digest);
    let mac = outer.finalize();

    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let mut mac_hex = String::with_capacity(mac.len() * 2);
    for b in mac {
        use std::fmt::Write;
        _ = write!(mac_hex, "{b:02x}");
    }
    format!(
        "{}:{}:{}:{mac_hex}",
        b64.encode(user_id.as_bytes()),
        b64.encode(version.to_string().as_bytes()),
        b64.encode(expires_unix.to_string().as_bytes())
    )
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

    let secret = systemprompt::config::SecretsBootstrap::manifest_signing_secret_seed()
        .expect("globals::init installed the fixture secrets");
    let now = chrono::Utc::now().timestamp();

    // Within the window: proves this signer matches the handler's verifier,
    // so the expired case below fails on the clock, not on a MAC mismatch.
    let live = sign(&secret, &user_id_str, 1, now + 3600);
    let (status, body) = app
        .call(Call::get(
            &format!("/share/manifest/{live}"),
            Principal::Anonymous,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "a token inside its window: {body}");

    // Same user, same version, same seed — only the expiry is in the past.
    let expired = sign(&secret, &user_id_str, 1, now - 60);
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

#[tokio::test]
async fn expiry_at_the_boundary_is_exclusive() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let user_id_str = seed::unique("share-boundary");
    let user_id = seed::insert_user(
        &db.pool,
        &user_id_str,
        &format!("{user_id_str}@contract.test"),
    )
    .await;
    seed::insert_profile_ext(&db.pool, &user_id, 1).await;

    let secret = systemprompt::config::SecretsBootstrap::manifest_signing_secret_seed()
        .expect("globals::init installed the fixture secrets");

    // A token whose expiry equals "now" must already be dead: the verifier
    // requires now strictly earlier than the expiry instant.
    let at_boundary = sign(&secret, &user_id_str, 1, chrono::Utc::now().timestamp());
    let (status, _) = app
        .call(Call::get(
            &format!("/share/manifest/{at_boundary}"),
            Principal::Anonymous,
        ))
        .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "a token at its expiry instant is already expired"
    );

    db.cleanup().await;
}
