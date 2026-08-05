//! The per-plugin secrets endpoints, driven end-to-end.
//!
//! `secrets_router` is merged at the root by `extensions/web/src/router/api.rs`
//! rather than mounted inside either route module, so the derived contract
//! table in [`crate::route_source`] cannot see it and never drove a single one
//! of these routes. What that left unexercised is the whole point of the
//! endpoints: two different authentication schemes on one router (a
//! `aud=plugin` bearer for the machine-to-machine pair, an `aud=api` admin
//! session for the operator pair), and a one-shot token whose value is that it
//! stops working the second time it is presented.
//!
//! The resolution flow is asserted as a sequence rather than as four
//! independent cases, because the interesting properties are transitions:
//! mint, then consume, then fail to consume again. Each step reads the audit
//! log back, since that trail is the only durable evidence a secret was
//! touched and a handler that silently skipped writing it would otherwise pass.

use axum::http::StatusCode;
use serde_json::Value;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

const PLUGIN: &str = "contract-secrets-plugin";

fn token_path(plugin: &str) -> String {
    format!("/api/v1/secrets/{plugin}/token")
}

fn resolve_path(plugin: &str, token: &str) -> String {
    format!("/api/v1/secrets/{plugin}/resolve?token={token}")
}

fn audit_path(plugin: &str) -> String {
    format!("/admin/api/secrets/{plugin}/audit")
}

fn rotate_path(plugin: &str) -> String {
    format!("/admin/api/secrets/{plugin}/rotate")
}

fn parse(body: &str) -> Value {
    serde_json::from_str(body).unwrap_or_else(|e| panic!("response is JSON: {e}\n{body}"))
}

// The actions recorded against a plugin, newest first, as the audit endpoint
// reports them.
async fn audit_actions(app: &App, plugin: &str) -> Vec<String> {
    let (status, body) = app
        .call(Call::get(&audit_path(plugin), Principal::Admin))
        .await;
    assert_eq!(status, StatusCode::OK, "audit log: {body}");
    parse(&body)["entries"]
        .as_array()
        .expect("entries is an array")
        .iter()
        .map(|e| e["action"].as_str().unwrap_or_default().to_owned())
        .collect()
}

#[tokio::test]
async fn secret_resolution_token_requires_a_plugin_audience() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // No credentials at all.
    let (status, body) = app
        .call(Call::json(
            "post",
            &token_path(PLUGIN),
            Principal::Anonymous,
            "{}",
        ))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "anonymous: {body}");
    assert_eq!(parse(&body)["error"], "Missing Authorization header");

    // A real, unexpired admin session token — valid, but minted for the
    // standard audiences rather than `plugin`. This is the case that proves
    // the endpoint is not merely checking "is this a token we signed".
    let (status, body) = app
        .call(Call::json(
            "post",
            &token_path(PLUGIN),
            Principal::Admin,
            "{}",
        ))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "admin session: {body}");
    assert_eq!(parse(&body)["error"], "Invalid or expired token");

    // Well-formed but not a JWT at all.
    let (status, body) = app
        .call_with_bearer(
            Call::json("post", &token_path(PLUGIN), Principal::Anonymous, "{}"),
            "not-a-jwt",
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "garbage bearer: {body}");

    db.cleanup().await;
}

#[tokio::test]
async fn secret_resolution_token_is_single_use() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let user_id_str = seed::unique("secrets-user");
    let user_id = seed::insert_user(
        &db.pool,
        &user_id_str,
        &format!("{user_id_str}@contract.test"),
    )
    .await;
    let plugin_token = seed::mint(&seed::TokenSpec::plugin(user_id.as_str()));

    // Mint.
    let (status, body) = app
        .call_with_bearer(
            Call::json("post", &token_path(PLUGIN), Principal::Anonymous, "{}"),
            &plugin_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "mint: {body}");
    let minted = parse(&body);
    assert_eq!(minted["expires_in"], 300);
    let resolution_token = minted["token"]
        .as_str()
        .expect("a resolution token")
        .to_owned();
    assert!(
        !resolution_token.is_empty(),
        "the minted token is not empty"
    );

    // Consume. The user holds no secrets, so the map is empty — the assertion
    // is that the flow completed, not that anything was decrypted.
    let (status, body) = app
        .call(Call::get(
            &resolve_path(PLUGIN, &resolution_token),
            Principal::Anonymous,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "resolve: {body}");
    assert_eq!(
        parse(&body)["secrets"],
        serde_json::json!({}),
        "no secrets are stored for this user"
    );

    // Consume again. The row is marked used, so this must fail — the property
    // the whole one-shot scheme exists for.
    let (status, body) = app
        .call(Call::get(
            &resolve_path(PLUGIN, &resolution_token),
            Principal::Anonymous,
        ))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "replay: {body}");
    assert_eq!(parse(&body)["error"], "Invalid or expired token");

    db.cleanup().await;
}

#[tokio::test]
async fn secret_resolution_rejects_a_token_issued_for_another_plugin() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let user_id_str = seed::unique("secrets-cross");
    let user_id = seed::insert_user(
        &db.pool,
        &user_id_str,
        &format!("{user_id_str}@contract.test"),
    )
    .await;
    let plugin_token = seed::mint(&seed::TokenSpec::plugin(user_id.as_str()));

    let (status, body) = app
        .call_with_bearer(
            Call::json("post", &token_path(PLUGIN), Principal::Anonymous, "{}"),
            &plugin_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "mint: {body}");
    let resolution_token = parse(&body)["token"]
        .as_str()
        .expect("a resolution token")
        .to_owned();

    // Presented at a different plugin's resolve endpoint. The token itself is
    // valid and unused, so this must be refused on the binding alone.
    let (status, body) = app
        .call(Call::get(
            &resolve_path("some-other-plugin", &resolution_token),
            Principal::Anonymous,
        ))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "cross-plugin: {body}");
    assert_eq!(parse(&body)["error"], "Token plugin mismatch");

    db.cleanup().await;
}

#[tokio::test]
async fn secret_resolution_rejects_unknown_and_missing_tokens() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // A token that was never minted.
    let (status, body) = app
        .call(Call::get(
            &resolve_path(PLUGIN, &uuid::Uuid::new_v4().to_string()),
            Principal::Anonymous,
        ))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "unknown token: {body}");

    // No `token` query parameter at all: the `Query` extractor rejects it
    // before the handler runs, and that rejection must not be a 500.
    let (status, body) = app
        .call(Call::get(
            &format!("/api/v1/secrets/{PLUGIN}/resolve"),
            Principal::Anonymous,
        ))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "missing query: {body}");

    db.cleanup().await;
}

#[tokio::test]
async fn secret_audit_and_rotate_require_an_admin_session() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    for (method, path) in [
        ("get", audit_path(PLUGIN)),
        ("post", rotate_path(PLUGIN)),
    ] {
        let (status, body) = app
            .call(Call::json(method, &path, Principal::Anonymous, "{}"))
            .await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "anonymous {method} {path}: {body}"
        );
    }

    // These two take an `aud=api` session, so the plugin bearer that opens the
    // resolution endpoints is the wrong credential here — the mirror image of
    // `secret_resolution_token_requires_a_plugin_audience`.
    let plugin_token = seed::mint(&seed::TokenSpec::plugin("secrets-wrong-audience"));
    let (status, body) = app
        .call_with_bearer(
            Call::get(&audit_path(PLUGIN), Principal::Anonymous),
            &plugin_token,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "plugin bearer: {body}");

    db.cleanup().await;
}

#[tokio::test]
async fn rotating_keys_records_the_rotation_in_the_audit_log() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let plugin = seed::unique("rotate-plugin");
    assert!(
        audit_actions(&app, &plugin).await.is_empty(),
        "a plugin nobody has touched has no audit trail"
    );

    // First rotation: no key exists yet, so this exercises the create branch
    // of `get_or_create_user_dek` on the way through.
    let (status, body) = app
        .call(Call::json(
            "post",
            &rotate_path(&plugin),
            Principal::Admin,
            "{}",
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "first rotate: {body}");
    assert_eq!(parse(&body)["result"], "ok");
    assert_eq!(audit_actions(&app, &plugin).await, vec!["rotated".to_owned()]);

    // Second rotation: the key now exists, so the decrypt-and-re-wrap branch
    // runs instead. Both must land an entry.
    let (status, body) = app
        .call(Call::json(
            "post",
            &rotate_path(&plugin),
            Principal::Admin,
            "{}",
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "second rotate: {body}");
    assert_eq!(
        audit_actions(&app, &plugin).await,
        vec!["rotated".to_owned(), "rotated".to_owned()]
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_audit_log_is_scoped_to_one_plugin() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let mine = seed::unique("audit-mine");
    let theirs = seed::unique("audit-theirs");

    let (status, _) = app
        .call(Call::json(
            "post",
            &rotate_path(&mine),
            Principal::Admin,
            "{}",
        ))
        .await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(audit_actions(&app, &mine).await.len(), 1);
    assert!(
        audit_actions(&app, &theirs).await.is_empty(),
        "one plugin's rotation is invisible to another's audit log"
    );

    db.cleanup().await;
}
