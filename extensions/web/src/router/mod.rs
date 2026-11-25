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
use tower::ServiceExt;

pub(crate) use pools::DbHandles;

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
    ("/api/public/secrets", "/api/public/secrets/{*rest}"),
    ("/api/public/connectors", "/api/public/connectors/{*rest}"),
    ("/api/public/bridge", "/api/public/bridge/{*rest}"),
    ("/share", "/share/{*rest}"),
];

/// The routers the web member serves, built once from its provided values.
#[derive(Clone)]
pub struct WebRuntime {
    router: Router,
}

impl std::fmt::Debug for WebRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebRuntime").finish_non_exhaustive()
    }
}

impl WebRuntime {
    pub(crate) fn build(db: &DbHandles, settings: &crate::WebSettings) -> Self {
        let session_service = pools::build_session_service(db);
        let sso_deps = crate::admin::AdfsDeps {
            config: crate::config_loader::adfs_config(
                &settings.services_path,
                &settings.api_external_url,
            )
            .unwrap_or_else(|| Arc::new(crate::admin::AdfsConfig::disabled())),
            write_pool: Arc::clone(&db.write),
            session_service: Arc::clone(&session_service),
        };
        let mut router = Router::new()
            .merge(api::share(db))
            .nest("/api/public", api::build(db, &session_service, settings));
        match admin_ssr::build(db, sso_deps, settings) {
            Some(ssr) => {
                router = router
                    .nest_service("/admin", ssr.admin)
                    .nest_service("/bridge-auth", ssr.bridge_auth);
            },
            None => tracing::error!(
                "Admin template engine failed to initialise; serving API routes without the SSR dashboard"
            ),
        }
        Self { router }
    }
}

/// Route-set state: the runtime the member provided.
#[derive(Debug, Clone)]
pub struct WebState(Arc<WebRuntime>);

impl FromState for WebState {
    fn from_state(r: &Resolved<'_>) -> Result<Self, MissingDependency> {
        Ok(Self(r.get::<Arc<WebRuntime>>()?))
    }
}

async fn forward(State(state): State<WebState>, request: Request<Body>) -> Response {
    match state.0.router.clone().oneshot(request).await {
        Ok(response) => response,
        Err(never) => match never {},
    }
    .into_response()
}

#[must_use]
pub fn route_set() -> RouteSet {
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
