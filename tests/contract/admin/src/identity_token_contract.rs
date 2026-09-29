//! The identity accessor signs only for the gateway broker, only for a server
//! that opted in, and only for a caller the access rules admit; the token it
//! returns verifies against the instance authority key with the agreed claims.

use axum::http::StatusCode;
use base64::Engine;
use jsonwebtoken::{Algorithm, Validation, decode};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin::identity_token::{IdentityClaims, TOKEN_TTL_SECS};

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

const ROUTE: &str = "/api/public/identity/partner_tools/token";
const AUDIENCE: &str = "https://mcp-dev-server-611435061062.asia-south1.run.app/mcp";
const BROKER: (&str, &str) = (
    "x-systemprompt-credential-broker",
    "contract-suite-broker-not-a-real-secret",
);

fn bearer(app_token: &str) -> String {
    format!("Bearer {app_token}")
}

// The principal token carries a session id; materialise it so the accessor's
// live-login guard passes and the case reaches the decision under test.
async fn live_session(pool: &PgPool, token: &str, user: &UserId) {
    let payload = token.split('.').nth(1).expect("jwt payload");
    let claims: serde_json::Value = serde_json::from_slice(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .expect("decode jwt payload"),
    )
    .expect("claims json");
    sqlx::query("INSERT INTO user_sessions (session_id, user_id) VALUES ($1, $2)")
        .bind(claims["session_id"].as_str().expect("session claim"))
        .bind(user.as_str())
        .execute(pool)
        .await
        .expect("insert live session");
}

#[tokio::test(flavor = "multi_thread")]
async fn identity_token_requires_the_credential_broker() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    for principal in [Principal::Anonymous, Principal::Admin] {
        let (status, body) = app.call(Call::get(ROUTE, principal)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{principal:?}: {body}");
        assert!(
            !body.contains("access_token"),
            "no token without the broker"
        );
    }
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn identity_token_is_signed_for_an_entitled_caller() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let admin = credentials.admin.clone();
    let admin_id = credentials.admin_user_id.clone();
    let non_admin = credentials.non_admin.clone();
    live_session(&db.pool, &admin, &admin_id).await;
    live_session(&db.pool, &non_admin, &credentials.non_admin_user_id).await;
    seed::insert_acl_rule(
        &db.pool,
        "mcp_server",
        "partner_tools",
        "role",
        "admin",
        "allow",
    )
    .await;
    let app = App::new(&db.pool, credentials);

    let (status, body) = app
        .call_with_headers(
            Call::get(ROUTE, Principal::Anonymous),
            &[BROKER, ("authorization", &bearer(&admin))],
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let token =
        serde_json::from_str::<serde_json::Value>(&body).expect("json body")["access_token"]
            .as_str()
            .expect("access_token")
            .to_owned();

    let header = jsonwebtoken::decode_header(&token).expect("jwt header");
    assert_eq!(header.alg, Algorithm::RS256);
    assert_eq!(
        header.kid.as_deref(),
        Some(systemprompt_security::keys::authority::active_kid().expect("kid"))
    );
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[AUDIENCE]);
    validation.set_issuer(&[systemprompt::models::Config::get()
        .expect("config")
        .jwt_issuer
        .as_str()]);
    let claims = decode::<IdentityClaims>(
        &token,
        systemprompt_security::keys::authority::decoding_key().expect("decoding key"),
        &validation,
    )
    .expect("token verifies against the authority key")
    .claims;
    assert_eq!(claims.sub, admin_id.as_str());
    assert!(!claims.email.is_empty(), "email claim is populated");
    assert!(!claims.name.is_empty(), "name claim is populated");
    assert_eq!(claims.exp - claims.iat, TOKEN_TTL_SECS);

    let (status, body) = app
        .call_with_headers(
            Call::get(ROUTE, Principal::Anonymous),
            &[BROKER, ("authorization", &bearer(&non_admin))],
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "non-entitled caller: {body}");
    assert!(!body.contains("access_token"));
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn identity_token_refuses_servers_that_did_not_opt_in() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let admin = credentials.admin.clone();
    live_session(&db.pool, &admin, &credentials.admin_user_id).await;
    let app = App::new(&db.pool, credentials);

    for server in ["atlassian", "systemprompt", "no_such_server"] {
        let (status, body) = app
            .call_with_headers(
                Call::get(
                    &format!("/api/public/identity/{server}/token"),
                    Principal::Anonymous,
                ),
                &[BROKER, ("authorization", &bearer(&admin))],
            )
            .await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{server}: {body}");
        assert!(!body.contains("access_token"));
    }
    db.cleanup().await;
}
