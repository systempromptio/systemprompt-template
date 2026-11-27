//! Connector consent and broker routes fail closed before any credential state
//! can be read or written.

use axum::http::StatusCode;
use systemprompt_web_admin::connector_oauth::{self, Grant, Provider};
use systemprompt_web_admin::repositories::users::connector_credentials;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::{globals, principal, tokens};
use template_test_common::TempDb;

#[tokio::test(flavor = "multi_thread")]
async fn connector_token_never_exposes_credentials_without_broker_authentication() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    for principal in [Principal::Anonymous, Principal::Admin] {
        let (status, body) = app
            .call(Call::get(
                "/api/public/connectors/atlassian/token",
                principal,
            ))
            .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{principal:?}: {body}");
        assert!(
            !body.contains("access_token"),
            "no credential material is returned"
        );
    }
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn connector_consent_callbacks_enforce_owner_binding_and_single_use() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let admin_token = credentials.admin.clone();
    let admin_user = credentials.admin_user_id.clone();
    let non_admin_token = credentials.non_admin.clone();
    let non_admin_user = credentials.non_admin_user_id.clone();
    let app = App::new(&db.pool, credentials);

    // The contract principal token carries a session id; materialise that
    // session so the connector's live-login guard reaches the consumed-state
    // checks instead of stopping at authentication.
    let session = tokens::session_of(&admin_token);
    sqlx::query("INSERT INTO user_sessions (session_id, user_id) VALUES ($1, $2)")
        .bind(session.as_str())
        .bind(admin_user.as_str())
        .execute(db.pool.as_ref())
        .await
        .expect("insert live connector session");
    let other_session = tokens::session_of(&non_admin_token);
    sqlx::query("INSERT INTO user_sessions (session_id, user_id) VALUES ($1, $2)")
        .bind(other_session.as_str())
        .bind(non_admin_user.as_str())
        .execute(db.pool.as_ref())
        .await
        .expect("insert other live connector session");
    let state = "contract-consent-state";
    let grant = Grant {
        user: admin_user.as_str().to_owned(),
        configuration_binding: String::new(),
        authorization_issuer: String::new(),
        token_auth_method: String::new(),
        provider: Provider::Atlassian,
        client: String::new(),
        client_secret: String::new(),
        verifier: String::new(),
        access_token: String::new(),
        refresh_token: None,
        expires_at: 0,
        token_endpoint: String::new(),
        generation: 0,
        session: Some(session.as_str().to_owned()),
        auth_method: String::new(),
        account_id: String::new(),
        account_name: String::new(),
        resource_id: String::new(),
        resource_name: String::new(),
        authorization_scheme: "Bearer".to_owned(),
    };
    let sealed = connector_oauth::seal(template_test_common::env::member_cipher().as_ref(), &grant)
        .expect("seal consent state");
    connector_credentials::save_state(
        &db.pool,
        &admin_user,
        Provider::Atlassian.slug(),
        state,
        &sealed,
    )
    .await
    .expect("save consent state");

    let (status, body) = app
        .call_with_bearer(
            Call::get(
                "/api/public/connectors/atlassian/callback?state=contract-consent-state&error=denied",
                Principal::NonAdmin,
            ),
            &non_admin_token,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "foreign callback: {body}");
    assert!(body.contains("expired or already consumed"));
    let (status, body) = app
        .call_with_bearer(
            Call::get(
                "/api/public/connectors/atlassian/callback?state=contract-consent-state&error=denied",
                Principal::Admin,
            ),
            &admin_token,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "callback: {body}");
    assert!(body.contains("Connector consent denied"));
    assert!(
        connector_credentials::consume_state(
            &db.pool,
            &admin_user,
            Provider::Atlassian.slug(),
            state
        )
        .await
        .expect("state read")
        .is_none(),
        "state is single-use"
    );

    db.cleanup().await;
}
