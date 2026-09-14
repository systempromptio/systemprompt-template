//! Personal access tokens and device enrolment.
//!
//! Every route here mints or destroys a long-lived credential, and the
//! exhaustive table drives each one exactly once — with an empty JSON body,
//! which validation rejects before any of this runs, and with an id that
//! exists nowhere, which the revokers turn into a `404` before touching a row.
//! What that leaves untested is everything the endpoints are for: the secret
//! actually being issued, the row actually being written, and the revoke
//! actually taking effect.
//!
//! Two properties get the most attention. A revoke is scoped to the caller —
//! a token id is guessable in a way a token is not, so a revoke that only
//! matched on id would let any signed-in user disable anyone's bridge. And a
//! secret is returned exactly once, at issue, because nothing stores it.

use axum::http::StatusCode;
use serde_json::Value;
use sqlx::PgPool;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

const PATS: &str = "/admin/devices/pats";
const ENROLL: &str = "/api/public/admin/management/devices";

fn parse(body: &str) -> Value {
    serde_json::from_str(body).unwrap_or_else(|e| panic!("response is JSON: {e}\n{body}"))
}

// Issue a PAT for the calling principal and return its id and secret.
async fn issue(app: &App, principal: Principal, name: &str) -> (String, String) {
    let (status, body) = app
        .call(Call::json(
            "post",
            PATS,
            principal,
            &format!(r#"{{"name":"{name}"}}"#),
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "issue pat: {body}");
    let issued = parse(&body);
    assert_eq!(issued["name"], name);
    (
        issued["id"].as_str().expect("an id").to_owned(),
        issued["secret"].as_str().expect("a secret").to_owned(),
    )
}

async fn active_keys_for(pool: &PgPool, id: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_api_keys WHERE id = $1 AND revoked_at IS NULL",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .expect("count active api keys")
}

#[tokio::test]
async fn issuing_a_pat_returns_the_secret_once_and_stores_only_its_prefix() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (id, secret) = issue(&app, Principal::Admin, "laptop").await;
    assert!(!secret.is_empty(), "a secret was issued");

    // The stored row must not be able to reproduce the secret. A prefix is
    // enough to identify a key in a UI; the rest never touches the table.
    let (prefix, stored): (String, Option<String>) =
        sqlx::query_as("SELECT key_prefix, key_hash FROM user_api_keys WHERE id = $1")
            .bind(&id)
            .fetch_one(db.pool.as_ref())
            .await
            .expect("the key row exists");

    assert!(!prefix.is_empty(), "the prefix is recorded for display");
    assert!(
        secret.starts_with(&prefix),
        "the prefix identifies the secret it came from"
    );
    assert!(
        stored.as_deref() != Some(secret.as_str()),
        "the secret itself is not stored in the clear"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn two_pats_issued_for_the_same_name_are_still_distinct_credentials() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (first_id, first_secret) = issue(&app, Principal::Admin, "laptop").await;
    let (second_id, second_secret) = issue(&app, Principal::Admin, "laptop").await;

    assert_ne!(first_id, second_id, "each issue is its own credential");
    assert_ne!(first_secret, second_secret);

    db.cleanup().await;
}

#[tokio::test]
async fn a_pat_can_only_be_revoked_by_the_user_who_holds_it() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // Somebody else's token. It is written directly because the endpoint that
    // mints one sits behind the `/admin` gate, so the suite's non-admin
    // principal cannot reach it — but a token owned by another user is
    // exactly the row the revoke must refuse to touch.
    let stranger_name = seed::unique("stranger");
    let stranger = seed::insert_user(
        &db.pool,
        &stranger_name,
        &format!("{stranger_name}@contract.test"),
    )
    .await;
    let stranger_key = seed::unique("key");
    sqlx::query(
        "INSERT INTO user_api_keys (id, user_id, name, key_prefix, key_hash)
         VALUES ($1, $2, 'their-laptop', $3, 'not-a-real-hash')",
    )
    .bind(&stranger_key)
    .bind(stranger.as_str())
    // `key_prefix` is a unique VARCHAR(32); the id is longer than that.
    .bind(&stranger_key[..24])
    .execute(db.pool.as_ref())
    .await
    .expect("insert the stranger's key");

    // The admin knows the id, and holds every role the plane grants. Authority
    // over the admin plane must still not extend to disabling another
    // person's credential through this route.
    let (status, body) = app
        .call(Call::json(
            "delete",
            &format!("{PATS}/{stranger_key}"),
            Principal::Admin,
            "{}",
        ))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "someone else's pat: {body}");
    assert_eq!(
        active_keys_for(&db.pool, &stranger_key).await,
        1,
        "and the token still works"
    );

    // Their own, they can.
    let (own_id, _) = issue(&app, Principal::Admin, "my-laptop").await;
    let own_path = format!("{PATS}/{own_id}");
    let (status, body) = app
        .call(Call::json("delete", &own_path, Principal::Admin, "{}"))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "owner revoke: {body}");
    assert_eq!(active_keys_for(&db.pool, &own_id).await, 0);

    // Revoking twice is a 404, not a second success: there is nothing left to
    // revoke, and reporting otherwise would hide a stale client.
    let (status, _) = app
        .call(Call::json("delete", &own_path, Principal::Admin, "{}"))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "double revoke");

    db.cleanup().await;
}

#[tokio::test]
async fn issuing_a_pat_rejects_a_body_it_cannot_read() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // Each of these is a different extractor rejection, and none of them is
    // allowed to be a 500.
    let cases = [
        ("no name field", r#"{"expires_at":null}"#),
        ("name is not a string", r#"{"name":42}"#),
        (
            "expires_at is not a timestamp",
            r#"{"name":"x","expires_at":"soon"}"#,
        ),
    ];

    for (label, body) in cases {
        let (status, response) = app
            .call(Call::json("post", PATS, Principal::Admin, body))
            .await;
        assert!(
            status.is_client_error(),
            "{label}: refused as a client error, got {status}: {response}"
        );
    }

    db.cleanup().await;
}

#[tokio::test]
async fn enrolling_a_device_is_admin_only_and_returns_its_credential() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let id = seed::unique("enrol-user");
    let user = seed::insert_user(&db.pool, &id, &format!("{id}@contract.test")).await;
    let payload =
        format!(r#"{{"user_id":"{id}","name":"build-box","platform":"linux","hostname":"ci-01"}}"#);

    let (status, body) = app
        .call(Call::json("post", ENROLL, Principal::NonAdmin, &payload))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "non-admin enrol: {body}");

    let (status, body) = app
        .call(Call::json("post", ENROLL, Principal::Admin, &payload))
        .await;
    assert_eq!(status, StatusCode::CREATED, "admin enrol: {body}");
    let enrolled = parse(&body);

    // Enrolment is on behalf of a third party, so the response must name whom
    // it enrolled — an admin acting for the wrong user would otherwise be
    // indistinguishable from success.
    assert_eq!(enrolled["user_id"], user.as_str());
    assert_eq!(enrolled["name"], "build-box");
    assert_eq!(enrolled["platform"], "linux");
    assert_eq!(enrolled["hostname"], "ci-01");
    assert!(
        enrolled["secret"].as_str().is_some_and(|s| !s.is_empty()),
        "the device is given a credential"
    );
    assert!(
        enrolled["enrolled_at"].as_str().is_some(),
        "and the moment it was granted is recorded"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn an_omitted_hostname_enrols_as_empty_rather_than_failing() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let id = seed::unique("enrol-nohost");
    seed::insert_user(&db.pool, &id, &format!("{id}@contract.test")).await;

    // A headless device may not report one. The field is optional and the
    // column is not nullable, so the handler's default is load-bearing.
    let (status, body) = app
        .call(Call::json(
            "post",
            ENROLL,
            Principal::Admin,
            &format!(r#"{{"user_id":"{id}","name":"headless","platform":"linux"}}"#),
        ))
        .await;
    assert_eq!(status, StatusCode::CREATED, "no hostname: {body}");
    assert_eq!(parse(&body)["hostname"], "");

    db.cleanup().await;
}

#[tokio::test]
async fn revoking_a_certificate_that_is_not_yours_is_a_404() {
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
            "delete",
            &format!("/admin/devices/certs/{}", seed::unique("cert")),
            Principal::Admin,
            "{}",
        ))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "unknown cert: {body}");
    assert_eq!(parse(&body)["error"], "cert not found");

    db.cleanup().await;
}

// The admin-plane revoker. It is a different endpoint from the self-service
// one above and a different rule: there is no `user_id` in its predicate, so
// the tests that matter are that it does reach a stranger's credential, that
// it is refused to everyone below the admin roles, and that a second attempt
// is a 404 rather than a silent success.
const ADMIN_PATS: &str = "/api/public/admin/devices/pats";

async fn insert_stranger_key(pool: &PgPool, label: &str) -> (String, String) {
    let name = seed::unique(label);
    let user = seed::insert_user(pool, &name, &format!("{name}@contract.test")).await;
    let key = seed::unique("key");
    sqlx::query(
        "INSERT INTO user_api_keys (id, user_id, name, key_prefix, key_hash)
         VALUES ($1, $2, 'their-laptop', $3, 'not-a-real-hash')",
    )
    .bind(&key)
    .bind(user.as_str())
    .bind(&key[..24])
    .execute(pool)
    .await
    .expect("insert the stranger's key");
    (user.as_str().to_owned(), key)
}

#[tokio::test]
async fn the_admin_plane_revokes_a_credential_it_does_not_own() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (_, key) = insert_stranger_key(&db.pool, "fleet-owner").await;
    let path = format!("{ADMIN_PATS}/{key}");

    // The console roles below `admin` read the fleet page; none of them may
    // disable what it lists.
    for principal in [Principal::NonAdmin, Principal::ProjectManager] {
        let (status, body) = app.call(Call::json("delete", &path, principal, "{}")).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{principal:?}: {body}");
    }
    assert_eq!(active_keys_for(&db.pool, &key).await, 1, "still active");

    let (status, body) = app
        .call(Call::json("delete", &path, Principal::Admin, "{}"))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "admin revoke: {body}");
    assert_eq!(active_keys_for(&db.pool, &key).await, 0);

    // Nothing left to revoke is a 404, so a stale console cannot report a
    // second success over a credential that was already gone.
    let (status, _) = app
        .call(Call::json("delete", &path, Principal::Admin, "{}"))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "double revoke");

    db.cleanup().await;
}

#[tokio::test]
async fn an_unknown_credential_kind_is_a_404_rather_than_a_500() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // `{kind}` is matched in the handler, not by the router, so an unknown
    // value has to be turned into a refusal rather than falling through to a
    // query against a table that does not exist.
    let (status, body) = app
        .call(Call::json(
            "delete",
            "/api/public/admin/devices/passports/anything",
            Principal::Admin,
            "{}",
        ))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "unknown kind: {body}");

    db.cleanup().await;
}
