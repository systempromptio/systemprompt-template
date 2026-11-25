//! HTTP surface of the web extension, contributed as one route set at the web
//! root.
//!
//! The dashboard, the desktop-bridge sign-in flow, the JSON/webhook API and
//! the share manifest are axum routers built once, in [`WebRuntime`], from the
//! values the member provides; the route set forwards each literal prefix to
//! its router. Every route is `public` at the mount: `/admin` and
//! `/bridge-auth` are gated by the member's site auth, and the API routers
//! authenticate each request themselves. There is no root fallback: the
//! public site is the generator's static output, served by core.

mod admin_ssr;
mod api;
mod pools;

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::Method;
use axum::response::{IntoResponse, Response};
use systemprompt::extension::RouteGroup;
use systemprompt::extension::prelude::*;
use systemprompt::extension::router::{
    BasePath, BodyLimit, RouteLimits, RouteSet, RouteSpec, WebBase,
};
use systemprompt::api::ServicesRefresh;
use systemprompt::governance::policy::GovernanceEngine;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::traits::ConfigWatch;
use tower::ServiceExt;

pub(crate) use pools::DbHandles;
use systemprompt_web_admin::env::AdminEnv;

const LIMITS: RouteLimits = RouteLimits {
    rate: RateLimit::new("web", RateBudget::Content),
    body: BodyLimit::Default,
};
const FORWARDED: RouteSpec = RouteSpec::new(AuthzPolicy::public(), ContextKind::Public, LIMITS);

const METHODS: [Method; 5] = [
    Method::GET,
    Method::POST,
    Method::PUT,
    Method::PATCH,
    Method::DELETE,
];

const PREFIXES: &[(&str, &str)] = &[
    ("/admin", "/admin/{*rest}"),
    ("/bridge-auth", "/bridge-auth/{*rest}"),
    ("/api/public/admin", "/api/public/admin/{*rest}"),
    (
        "/api/public/auth/session",
        "/api/public/auth/session/{*rest}",
    ),
    ("/api/public/links", "/api/public/links/{*rest}"),
    ("/api/public/search", "/api/public/search/{*rest}"),
    ("/api/public/hooks", "/api/public/hooks/{*rest}"),
    ("/api/public/govern", "/api/public/govern/{*rest}"),
    ("/api/public/identity", "/api/public/identity/{*rest}"),
    ("/api/public/account", "/api/public/account/{*rest}"),
    (
        "/api/public/api/v1/secrets",
        "/api/public/api/v1/secrets/{*rest}",
    ),
    ("/api/public/secrets", "/api/public/secrets/{*rest}"),
    ("/api/public/connectors", "/api/public/connectors/{*rest}"),
    ("/api/public/bridge", "/api/public/bridge/{*rest}"),
    ("/share", "/share/{*rest}"),
];

/// The routers the web member serves, built once from its provided values.
#[derive(Clone)]
pub(crate) struct WebRuntime {
    router: Router,
}

impl std::fmt::Debug for WebRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebRuntime").finish_non_exhaustive()
    }
}

impl WebRuntime {
    pub(crate) fn build(db: &DbHandles, settings: &crate::WebSettings, env: &AdminEnv) -> Self {
        let sso_deps = crate::admin::AdfsDeps {
            config: crate::config_loader::adfs_config(
                &settings.services_path,
                &settings.api_external_url,
            )
            .unwrap_or_else(|| Arc::new(crate::admin::AdfsConfig::disabled())),
            write_pool: Arc::clone(&db.write),
            use_https: settings.use_https,
        };
        let mut router = Router::new()
            .merge(api::share(db))
            .nest("/api/public", api::build(db, settings));
        match admin_ssr::build(db, sso_deps, settings, env) {
            Some(ssr) => {
                router = router
                    .nest_service("/admin", ssr.admin)
                    .nest_service("/bridge-auth", ssr.bridge_auth);
            },
            None => tracing::error!(
                "Admin template engine failed to initialise; serving API routes without the SSR dashboard"
            ),
        }
        Self {
            router: router.layer(axum::Extension(env.clone())),
        }
    }
}

/// Route-set state: the runtime the member provided.
#[derive(Debug, Clone)]
pub(crate) struct WebState(Arc<WebRuntime>);

impl FromState for WebState {
    fn from_state(r: &Resolved<'_>) -> Result<Self, MissingDependency> {
        Ok(Self(r.get::<Arc<WebRuntime>>()?))
    }
}

// Why: the route set's own match leaves its `MatchedPath` and its `{*rest}`
// capture on the request, and the forwarded routers match again: axum
// asserts a nested router never inherits a matched path, and an inner
// `Path` extractor would see the outer capture too. The request is rebuilt
// with a fresh extension map carrying only what the mount layered on for the
// web member's handlers.
async fn forward(State(state): State<WebState>, request: Request<Body>) -> Response {
    let (mut parts, body) = request.into_parts();
    let mut carried = axum::http::Extensions::new();
    if let Some(governance) = parts
        .extensions
        .remove::<ConfigWatch<GovernanceEngine>>()
    {
        carried.insert(governance);
    }
    if let Some(refresh) = parts.extensions.remove::<ServicesRefresh>() {
        carried.insert(refresh);
    }
    if let Some(context) = parts.extensions.remove::<RequestContext>() {
        carried.insert(context);
    }
    if let Some(peer) = parts
        .extensions
        .remove::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
    {
        carried.insert(peer);
    }
    parts.extensions = carried;
    let request = Request::from_parts(parts, body);
    match state.0.router.clone().oneshot(request).await {
        Ok(response) => response,
        Err(never) => match never {},
    }
    .into_response()
}

#[must_use]
pub(crate) fn route_set() -> RouteSet {
    let mut set = RouteSet::at::<WebState>(BasePath::Web(WebBase::Root), RouteGroup::Extensions);
    for (exact, nested) in PREFIXES {
        for method in METHODS {
            set = set
                .route(method.clone(), exact, forward, FORWARDED)
                .route(method, nested, forward, FORWARDED);
        }
    }
    set.build()
}
