//! The router under test, mounted at the prefixes the server mounts it at.
//!
//! This mirrors `extensions/web/src/extension_impl.rs` rather than calling the
//! per-group constructors directly: prefix handling is part of the contract.
//! `non_admin_gate_middleware` matches on `/admin/...` paths, and the SSR
//! router is attached with `nest_service`, so testing the groups in isolation
//! would exercise a path shape no request ever has.

use std::sync::Arc;

use systemprompt::analytics::AnalyticsService;
use systemprompt::analytics::repository::AnalyticsRepositories;
use systemprompt::database::Database;
use systemprompt::oauth::SessionCreationService;
use systemprompt::users::{UserRepository, UserService};

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt as _;
use sqlx::PgPool;
use systemprompt_web_admin as admin;
use tower::ServiceExt;

use crate::globals;
use crate::principal::{Credentials, Principal};

// Mount prefixes, kept next to the router build so the contract table and the
// exhaustiveness check agree on where each route module lands.
pub const ADMIN_API_PREFIX: &str = "/api/public/admin";
pub const SSR_PREFIX: &str = "/admin";

pub struct App {
    router: Router,
    credentials: Credentials,
}

fn session_service(pool: &Arc<PgPool>) -> Arc<SessionCreationService> {
    let db = Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ));
    let user = UserService::new(Arc::new(
        UserRepository::new(&db).expect("build the user repository"),
    ));
    let analytics_repos =
        AnalyticsRepositories::new(&db).expect("build the analytics repositories");
    let analytics = AnalyticsService::new(None, None, &analytics_repos);
    Arc::new(SessionCreationService::new(
        Arc::new(analytics),
        Arc::new(user),
    ))
}

impl App {
    pub fn new(pool: &Arc<PgPool>, credentials: Credentials) -> Self {
        Self::build(pool, credentials, admin::SalesforceConfig::disabled())
    }

    // The same router with Salesforce SSO configured.
    //
    // `new` disables it, which pins every SSO route at the "unavailable"
    // redirect and leaves the flow itself — PKCE, the state cookie, the
    // callback's validation ladder — unexercised. The config passed here
    // points at an address that refuses connections, so the branches before
    // and around the network call are reachable without a live org and
    // without the suite ever talking to Salesforce.
    pub fn with_salesforce(
        pool: &Arc<PgPool>,
        credentials: Credentials,
        config: admin::SalesforceConfig,
    ) -> Self {
        Self::build(pool, credentials, config)
    }

    fn build(
        pool: &Arc<PgPool>,
        credentials: Credentials,
        salesforce: admin::SalesforceConfig,
    ) -> Self {
        let admin_dir = globals::repo_root().join("storage/files/admin");
        // Branding is not decoration here: the templates read `branding.*`
        // under strict mode, so an engine built without it 500s on every page
        // the server renders fine.
        let branding = systemprompt_web_extension::branding_config();
        let engine = admin::templates::AdminTemplateEngine::new(&admin_dir)
            .expect("build the admin template engine from storage/files/admin")
            .with_branding(branding);

        let api = Router::new().nest("/admin", admin::admin_router(Arc::clone(pool), pool));
        // Salesforce SSO is disabled by default: the suite drives routes, and a
        // configured IdP would make every sign-in path depend on a live org.
        let sf_deps = admin::SalesforceDeps {
            config: Arc::new(salesforce),
            write_pool: Arc::clone(pool),
            session_service: session_service(pool),
        };
        let ssr = admin::admin_ssr_router(Arc::clone(pool), engine.clone(), sf_deps.clone());
        let bridge_auth = admin::bridge_auth_ssr_router(Arc::clone(pool), engine);

        // The hook endpoints are mounted at the root by
        // `extension_impl.rs`, outside both route modules the contract table
        // is derived from. They are here because `handler_errors` drives their
        // rejection paths; nothing reads them back into the table.
        let hooks = admin::hooks_webhook_router(Arc::clone(pool), session_service(pool));

        // `secrets_router` and `share_manifest_router` are merged at the root
        // by `extensions/web/src/router/api.rs`, not nested under either route
        // module, so `route_source` never sees them. They are mounted here at
        // the same prefixes the server uses so the secret-resolution flow and
        // the public manifest verifier are reachable from the suite.
        let secrets = admin::secrets_router(Arc::clone(pool));
        let share = admin::share_manifest_router(Arc::clone(pool));

        let router = Router::new()
            .nest_service(SSR_PREFIX, ssr)
            .nest_service("/bridge-auth", bridge_auth)
            .nest("/api/public", api)
            .merge(hooks)
            .merge(secrets)
            .merge(share);

        Self {
            router,
            credentials,
        }
    }

    // Issue one request, returning its status and — only when the status is a
    // server error — a snippet of the body.
    //
    // A contract failure that reports `500` and nothing else is barely
    // actionable, and the whole point of the suite is that a 5xx is a defect
    // someone has to go and fix.
    pub async fn send(
        &self,
        method: &str,
        path: &str,
        principal: Principal,
    ) -> (StatusCode, Option<String>) {
        // HTTP methods are case-sensitive; the route source spells them
        // lowercase after axum's constructors.
        let mut builder = Request::builder()
            .method(method.to_uppercase().as_str())
            .uri(path);
        if let Some(token) = self.credentials.token_for(principal) {
            builder = builder.header("authorization", format!("Bearer {token}"));
        }
        // Every write route takes JSON; an empty object is the most benign
        // well-formed body, and a 4xx from validation is a legitimate contract
        // outcome. What must not happen is a 500.
        let request = builder
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .expect("build request");

        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("router is infallible");
        let status = response.status();
        if !status.is_server_error() {
            return (status, None);
        }

        let bytes = response
            .into_body()
            .collect()
            .await
            .map(http_body_util::Collected::to_bytes)
            .unwrap_or_default();
        let snippet: String = String::from_utf8_lossy(&bytes).chars().take(300).collect();
        (status, Some(snippet))
    }

    // Issue one fully-specified request and return its status with the whole
    // body.
    //
    // `send` is deliberately narrow — one well-formed shape per route, body
    // read only on failure — because that is what the exhaustive table needs.
    // The variant and error suites need the opposite: a chosen query string, a
    // chosen payload, a chosen content type, and the rendered body on every
    // response, because the assertion *is* about which branch rendered.
    pub async fn call(&self, call: Call<'_>) -> (StatusCode, String) {
        self.dispatch(call, None, &[]).await
    }

    // Issue a call bearing a token this harness did not mint.
    //
    // The hook and webhook endpoints authenticate against audiences no
    // principal in [`Credentials`] holds — a hook token carries `aud=hook`
    // and a `plugin_id` claim, which the admin session token never does. The
    // token is therefore passed per call rather than resolved from the
    // principal, which also lets a case present one that is deliberately
    // wrong.
    pub async fn call_with_bearer(&self, call: Call<'_>, token: &str) -> (StatusCode, String) {
        self.dispatch(call, Some(token), &[]).await
    }

    // The redirect target of a call, for the flows whose whole contract is
    // where they send the browser.
    pub async fn redirect_of(&self, call: Call<'_>) -> (StatusCode, String) {
        self.redirect_with_headers(call, &[]).await
    }

    pub async fn redirect_with_headers(
        &self,
        call: Call<'_>,
        headers: &[(&str, &str)],
    ) -> (StatusCode, String) {
        let (status, headers) = self.response_headers_with(call, headers).await;
        (status, headers.location.unwrap_or_default())
    }

    pub async fn response_headers(&self, call: Call<'_>) -> (StatusCode, ResponseHeaders) {
        self.response_headers_with(call, &[]).await
    }

    // Status plus the two response headers the redirect-driven flows are
    // specified in terms of.
    //
    // `call` reads the body, which is empty on a redirect: for the SSO flows
    // the entire outcome — where the browser goes, and what state it carries
    // there — lives in `Location` and `Set-Cookie`.
    pub async fn response_headers_with(
        &self,
        call: Call<'_>,
        extra_headers: &[(&str, &str)],
    ) -> (StatusCode, ResponseHeaders) {
        let request = self.build_request(call, None, extra_headers);
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("router is infallible");
        let status = response.status();
        let headers = response.headers();
        let location = headers
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned);
        let set_cookie = headers
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(ToOwned::to_owned)
            .collect();
        (
            status,
            ResponseHeaders {
                location,
                set_cookie,
            },
        )
    }

    async fn dispatch(
        &self,
        call: Call<'_>,
        bearer: Option<&str>,
        extra_headers: &[(&str, &str)],
    ) -> (StatusCode, String) {
        let request = self.build_request(call, bearer, extra_headers);
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("router is infallible");
        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .map(http_body_util::Collected::to_bytes)
            .unwrap_or_default();
        (status, String::from_utf8_lossy(&bytes).into_owned())
    }

    fn build_request(
        &self,
        call: Call<'_>,
        bearer: Option<&str>,
        extra_headers: &[(&str, &str)],
    ) -> Request<Body> {
        let mut builder = Request::builder()
            .method(call.method.to_uppercase().as_str())
            .uri(call.path);
        match bearer {
            Some(token) => {
                builder = builder.header("authorization", format!("Bearer {token}"));
            },
            None => {
                if let Some(token) = self.credentials.token_for(call.principal) {
                    builder = builder.header("authorization", format!("Bearer {token}"));
                }
            },
        }
        for (name, value) in extra_headers {
            builder = builder.header(*name, *value);
        }
        if let Some(content_type) = call.content_type {
            builder = builder.header("content-type", content_type);
        }
        let body = call
            .body
            .map_or_else(Body::empty, |b| Body::from(b.to_owned()));
        builder.body(body).expect("build request")
    }
}

// The response headers the redirect-driven flows are specified in terms of.
pub struct ResponseHeaders {
    pub location: Option<String>,
    pub set_cookie: Vec<String>,
}

// One request, spelled out.
pub struct Call<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub principal: Principal,
    // `None` sends no `content-type` at all, which is itself a rejection path
    // worth driving: axum's `Json` extractor refuses a body it was not told
    // the type of.
    pub content_type: Option<&'a str>,
    pub body: Option<&'a str>,
}

impl<'a> Call<'a> {
    // A page fetch: no body, no content type.
    pub const fn get(path: &'a str, principal: Principal) -> Self {
        Self {
            method: "get",
            path,
            principal,
            content_type: None,
            body: None,
        }
    }

    // A JSON write, with the content type the extractor expects.
    pub const fn json(method: &'a str, path: &'a str, principal: Principal, body: &'a str) -> Self {
        Self {
            method,
            path,
            principal,
            content_type: Some("application/json"),
            body: Some(body),
        }
    }
}
