//! Who the console admits: a signed-in principal and nobody else.
//!
//! Core mints a verified anonymous session for every first-time visitor, and
//! its account row carries the `anonymous` role. Neither that token nor any
//! token whose account holds no console role may reach a console page or the
//! console API; an expired token is refused like any unverifiable one. The
//! request authenticator is a table of fixed claims, so each case states the
//! credential exactly; the account rows are real, because the middleware reads
//! roles from the database.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use systemprompt::database::Database;
use systemprompt::identifiers::{JwtToken, SessionId, UserId};
use systemprompt::models::auth::{
    DynRequestAuthenticator, JwtAudience, JwtUserContext, Permission, RequestAuthenticator,
};
use systemprompt::models::execution::context::{ContextExtractionError, RequestContext};
use systemprompt::traits::AuthUser;
use systemprompt_web_admin as admin;
use template_test_common::TempDb;
use template_test_common::env::EnvFixture;
use template_test_common::paths::repo_root;
use tower::ServiceExt as _;

use crate::fixtures::{AclRuleSpec, insert_acl_rule, insert_user_full, unique};

const ANONYMOUS: &str = "anonymous-session";
const ANONYMOUS_ROW: &str = "user-typed-token-on-an-anonymous-row";
const EXPIRED: &str = "expired-session";
const MEMBER: &str = "member-session";
const ADMIN: &str = "admin-session";

#[derive(Debug)]
struct TokenTable(HashMap<&'static str, (JwtUserContext, AuthUser)>);

fn refused(token: &JwtToken) -> ContextExtractionError {
    ContextExtractionError::InvalidToken(format!("{} is expired or unknown", token.as_str()).into())
}

#[async_trait::async_trait]
impl RequestAuthenticator for TokenTable {
    async fn authenticate(
        &self,
        _headers: &axum::http::HeaderMap,
        _audiences: &[JwtAudience],
    ) -> Result<RequestContext, ContextExtractionError> {
        Err(ContextExtractionError::MissingAuthHeader)
    }

    async fn decode_for_audiences(
        &self,
        token: &JwtToken,
        _audiences: &[JwtAudience],
    ) -> Result<(JwtUserContext, AuthUser), ContextExtractionError> {
        self.0
            .get(token.as_str())
            .cloned()
            .ok_or_else(|| refused(token))
    }

    async fn decode_for_gateway(
        &self,
        token: &JwtToken,
    ) -> Result<(JwtUserContext, AuthUser), ContextExtractionError> {
        Err(refused(token))
    }

    async fn load_user(
        &self,
        _user_id: &UserId,
        _route_context: &str,
    ) -> Result<AuthUser, ContextExtractionError> {
        Err(ContextExtractionError::MissingAuthHeader)
    }
}

fn claims(
    user_id: &UserId,
    email: &str,
    permission: Permission,
    roles: &[&str],
) -> (JwtUserContext, AuthUser) {
    let context = JwtUserContext {
        user_id: user_id.clone(),
        session_id: SessionId::new(uuid::Uuid::new_v4().to_string()),
        role: permission,
        scope: vec![permission],
        user_type: permission.as_user_type(),
        client_id: None,
        act_chain: Vec::new(),
        attributes: BTreeMap::new(),
        jti: None,
        exp: chrono::Utc::now().timestamp() + 3600,
        purpose: None,
        interaction: None,
        browser_session: true,
        ui_session: false,
        auth_time: chrono::Utc::now().timestamp(),
    };
    let user = AuthUser {
        id: user_id.clone(),
        name: user_id.as_str().to_owned(),
        email: email.to_owned(),
        roles: roles.iter().map(|r| (*r).to_owned()).collect(),
        is_active: true,
        is_service_account: false,
    };
    (context, user)
}

async fn account(db: &TempDb, prefix: &str, roles: &[&str]) -> (UserId, String) {
    let id = unique(prefix);
    let email = format!("{id}@principal.test");
    let roles: Vec<String> = roles.iter().map(|r| (*r).to_owned()).collect();
    let user_id = insert_user_full(&db.pool, &id, &email, None, &roles).await;
    (user_id, email)
}

async fn console(db: &TempDb) -> Router {
    let (anon, anon_email) = account(db, "fp-anon", &["anonymous"]).await;
    let (member, member_email) = account(db, "member", &["user"]).await;
    let (operator, operator_email) = account(db, "operator", &["admin", "user"]).await;
    let table = TokenTable(HashMap::from([
        (
            ANONYMOUS,
            claims(&anon, &anon_email, Permission::Anonymous, &["anonymous"]),
        ),
        (
            ANONYMOUS_ROW,
            claims(&anon, &anon_email, Permission::User, &["anonymous"]),
        ),
        (
            MEMBER,
            claims(&member, &member_email, Permission::User, &["user"]),
        ),
        (
            ADMIN,
            claims(
                &operator,
                &operator_email,
                Permission::Admin,
                &["admin", "user"],
            ),
        ),
    ]));
    let authenticator: DynRequestAuthenticator = Arc::new(table);
    let fixture = EnvFixture::with_authenticator(&db.pool, authenticator);
    let database = Arc::new(Database::from_pools(
        Arc::clone(&db.pool),
        Some(Arc::clone(&db.pool)),
    ));
    let engine =
        admin::templates::AdminTemplateEngine::new(&repo_root().join("storage/files/admin"))
            .expect("build the admin template engine")
            .with_branding(systemprompt_web_shared::config_loader::branding_config(
                &repo_root().join("services"),
            ));
    let ssr = admin::admin_ssr_router(
        &database,
        Arc::clone(&db.pool),
        operator.clone(),
        admin::AdminSsrShell {
            engine,
            sign_in: admin::SignInOptions::default(),
        },
    );
    let api = admin::admin_router(&database, Arc::clone(&db.pool), &db.pool, operator);
    let session = admin::session_router(Arc::clone(&db.pool), true);
    Router::new()
        .nest_service("/admin", ssr)
        .nest("/api", api)
        .merge(session)
        .layer(axum::Extension(fixture.env))
}

async fn send(router: &Router, request: Request<Body>) -> (StatusCode, Vec<String>) {
    let response = router
        .clone()
        .oneshot(request)
        .await
        .expect("the router is infallible");
    let cookies = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().expect("cookies are ASCII").to_owned())
        .collect();
    (response.status(), cookies)
}

fn set_session(token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/auth/session")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({ "access_token": token }).to_string(),
        ))
        .expect("build the request")
}

async fn get(router: &Router, path: &str, token: Option<&str>) -> (StatusCode, String) {
    let mut builder = Request::builder().uri(path);
    if let Some(token) = token {
        builder = builder.header("cookie", format!("access_token={token}"));
    }
    let response = router
        .clone()
        .oneshot(builder.body(Body::empty()).expect("build the request"))
        .await
        .expect("the router is infallible");
    let location = response
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    (response.status(), location)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_session_that_is_not_signed_in_is_sent_to_sign_in_and_refused_by_the_api() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let router = console(&db).await;

    for token in [None, Some(ANONYMOUS), Some(ANONYMOUS_ROW), Some(EXPIRED)] {
        let (status, location) = get(&router, "/admin/profile", token).await;
        assert_eq!(status, StatusCode::TEMPORARY_REDIRECT, "{token:?}");
        assert_eq!(
            location, "/admin/login?redirect=%2Fadmin%2Fprofile",
            "{token:?}"
        );

        let (status, location) =
            get(&router, "/admin/login?redirect=%2Fadmin%2Fusers", token).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "{token:?} must see the sign-in form, not {location}"
        );

        let (status, _) = get(&router, "/admin/auth/me", token).await;
        assert_eq!(status, StatusCode::TEMPORARY_REDIRECT, "{token:?}");

        let (status, _) = get(&router, "/api/dashboard", token).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{token:?}");
    }

    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_signed_in_member_is_admitted_to_their_pages_and_kept_out_of_the_console_data() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let router = console(&db).await;

    let (status, location) = get(&router, "/admin/login", Some(MEMBER)).await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(location, "/admin/profile");

    let (status, _) = get(&router, "/admin/auth/me", Some(MEMBER)).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = get(&router, "/api/dashboard", Some(MEMBER)).await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "the estate-wide dashboard is console data"
    );

    let (status, location) = get(&router, "/admin/login", Some(ADMIN)).await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(location, "/admin");

    let (status, _) = get(&router, "/admin/auth/me", Some(ADMIN)).await;
    assert_eq!(status, StatusCode::OK);

    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_session_cookie_is_set_only_for_a_signed_in_principal() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let router = console(&db).await;

    let (status, cookies) = send(&router, set_session(MEMBER)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cookies.len(), 1);
    let cookie = &cookies[0];
    assert!(
        cookie.starts_with(&format!("access_token={MEMBER}; ")),
        "{cookie}"
    );
    for attribute in ["HttpOnly", "SameSite=Lax", "Secure", "Path=/"] {
        assert!(
            cookie.contains(attribute),
            "{attribute} missing from {cookie}"
        );
    }
    let max_age: i64 = cookie
        .split("; ")
        .find_map(|part| part.strip_prefix("Max-Age="))
        .and_then(|value| value.parse().ok())
        .expect("a Max-Age attribute");
    assert!(
        (3500..=3600).contains(&max_age),
        "the lifetime is the token's own expiry, not a client-chosen number: {max_age}"
    );

    for token in [
        ANONYMOUS,
        ANONYMOUS_ROW,
        EXPIRED,
        "x; Domain=.evil.example; Max-Age=99999999",
    ] {
        let (status, cookies) = send(&router, set_session(token)).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{token}");
        assert!(
            cookies.is_empty(),
            "{token} must set no cookie: {cookies:?}"
        );
    }

    let unknown_field = Request::builder()
        .method("POST")
        .uri("/auth/session")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({ "access_token": MEMBER, "expires_in": 99_999_999 }).to_string(),
        ))
        .expect("build the request");
    let (status, cookies) = send(&router, unknown_field).await;
    assert!(status.is_client_error(), "{status}");
    assert!(cookies.is_empty());

    let clear = Request::builder()
        .method("DELETE")
        .uri("/auth/session")
        .body(Body::empty())
        .expect("build the request");
    let (status, cookies) = send(&router, clear).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cookies.len(), 1);
    assert!(cookies[0].starts_with("access_token=; "), "{}", cookies[0]);
    assert!(cookies[0].contains("Max-Age=0"), "{}", cookies[0]);

    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn an_agent_configuration_is_shown_only_to_a_caller_granted_the_agent() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let router = console(&db).await;
    let path = "/api/agents/developer_agent";

    let (status, _) = get(&router, path, Some(MEMBER)).await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "an ungranted agent is indistinguishable from an absent one"
    );
    let (status, _) = get(&router, path, Some(ADMIN)).await;
    assert_eq!(status, StatusCode::OK, "an admin sees every agent");

    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::allow("agent", "developer_agent", "role", "user"),
    )
    .await;
    let (status, _) = get(&router, path, Some(MEMBER)).await;
    assert_eq!(status, StatusCode::OK, "a granted member sees the agent");

    let (status, _) = get(&router, "/api/agents/associate_agent", Some(MEMBER)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "the grant is per agent");

    let (status, _) = get(&router, path, Some(ANONYMOUS)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    db.cleanup().await;
}
