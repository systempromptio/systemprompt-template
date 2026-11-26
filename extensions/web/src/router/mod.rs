//! HTTP surface of the web extension, contributed as one route set at the web
//! root.
//!
//! The dashboard, the desktop-bridge sign-in flow, the JSON/webhook API and
//! the share manifest are axum routers built once, in [`WebRuntime`], from the
//! values the member provides. Each prefix the member serves is a late-built
//! service (`RouteSetBuilder::service_with`): the API materialises it from the
//! route set's own state, so the request reaches the member's router with
//! every extension the mount layered on. Every service is `public` at the
//! mount: `/admin` and `/bridge-auth` are gated by the member's site auth,
//! and the API routers authenticate each request themselves. There is no root
//! route: `/` is the kernel's homepage, the generator's `index.html` with
//! `content` and the console landing page (`crate::extension_impl`) without
//! it.

mod admin_ssr;
mod api;
mod pools;

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, Uri};
use systemprompt::extension::RouteGroup;
use systemprompt::extension::prelude::*;
use systemprompt::extension::router::{
    BasePath, BodyLimit, RouteLimits, RouteSet, RouteSpec, WebBase,
};
use tower::ServiceExt;
use tower::util::MapRequest;

pub(crate) use pools::DbHandles;
use systemprompt_web_admin::SignInOptions;
use systemprompt_web_admin::dashboard::ConsoleDashboard;
use systemprompt_web_admin::env::AdminEnv;

const LIMITS: RouteLimits = RouteLimits {
    rate: RateLimit::new("web", RateBudget::Content),
    body: BodyLimit::Default,
};
const SERVED: RouteSpec = RouteSpec::new(AuthzPolicy::public(), ContextKind::Public, LIMITS);

const PUBLIC_API: &str = "/api/public";

// Why: core serves `/api/public/gateway`, so the member claims each of its
// own `/api/public` subtrees, not the whole prefix.
const PUBLIC_API_PREFIXES: &[&str] = &[
    "/api/public/admin",
    "/api/public/auth/session",
    #[cfg(feature = "content")]
    "/api/public/links",
    #[cfg(feature = "content")]
    "/api/public/search",
    "/api/public/hooks",
    "/api/public/govern",
    "/api/public/account",
    "/api/public/api/v1/secrets",
    "/api/public/secrets",
    #[cfg(feature = "mcp")]
    "/api/public/connectors",
    "/api/public/bridge",
];

type Rebased = MapRequest<Router, fn(Request<Body>) -> Request<Body>>;

#[derive(Clone)]
pub(crate) struct WebRuntime {
    admin: Option<Router>,
    bridge_auth: Option<Router>,
    public_api: Router,
    share: Router,
}

impl std::fmt::Debug for WebRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebRuntime").finish_non_exhaustive()
    }
}

impl WebRuntime {
    pub(crate) fn build(
        db: &DbHandles,
        settings: &crate::WebSettings,
        env: &AdminEnv,
        dashboard: Arc<ConsoleDashboard>,
        sign_in: SignInOptions,
    ) -> Self {
        let layer = axum::Extension(env.clone());
        let ssr = admin_ssr::build(db, settings, dashboard, sign_in);
        if ssr.is_none() {
            tracing::error!(
                "Admin template engine failed to initialise; serving API routes without the SSR dashboard"
            );
        }
        let (admin, bridge_auth) = ssr.map_or((None, None), |ssr| {
            (
                Some(ssr.admin.layer(layer.clone())),
                Some(ssr.bridge_auth.layer(layer.clone())),
            )
        });
        Self {
            admin,
            bridge_auth,
            public_api: api::build(db, settings).layer(layer.clone()),
            share: api::share(db).layer(layer),
        }
    }

    fn admin(&self) -> Router {
        self.admin.clone().unwrap_or_default()
    }

    fn bridge_auth(&self) -> Router {
        self.bridge_auth.clone().unwrap_or_default()
    }

    // Why: the route set strips the claimed prefix before the service sees the
    // request, and the public API router routes on paths below `/api/public`,
    // so the part of the claimed prefix below that base is put back.
    fn public_api(&self, prefix: &'static str) -> Rebased {
        let rebase: fn(Request<Body>) -> Request<Body> = match prefix {
            "/api/public/admin" => |r| rebase(r, "/admin"),
            "/api/public/auth/session" => |r| rebase(r, "/auth/session"),
            #[cfg(feature = "content")]
            "/api/public/links" => |r| rebase(r, "/links"),
            #[cfg(feature = "content")]
            "/api/public/search" => |r| rebase(r, "/search"),
            "/api/public/hooks" => |r| rebase(r, "/hooks"),
            "/api/public/govern" => |r| rebase(r, "/govern"),
            "/api/public/account" => |r| rebase(r, "/account"),
            "/api/public/api/v1/secrets" => |r| rebase(r, "/api/v1/secrets"),
            "/api/public/secrets" => |r| rebase(r, "/secrets"),
            #[cfg(feature = "mcp")]
            "/api/public/connectors" => |r| rebase(r, "/connectors"),
            _ => |r| rebase(r, "/bridge"),
        };
        self.public_api.clone().map_request(rebase)
    }
}

fn rebase(mut request: Request<Body>, below: &str) -> Request<Body> {
    let uri = request.uri();
    let path = uri.path();
    let tail = if path == "/" { "" } else { path };
    let rebuilt = uri.query().map_or_else(
        || format!("{below}{tail}"),
        |query| format!("{below}{tail}?{query}"),
    );
    if let Ok(rebuilt) = rebuilt.parse::<Uri>() {
        *request.uri_mut() = rebuilt;
    }
    request
}

#[derive(Debug, Clone)]
pub(crate) struct WebState(Arc<WebRuntime>);

impl FromState for WebState {
    fn from_state(r: &Resolved<'_>) -> Result<Self, MissingDependency> {
        Ok(Self(r.get::<Arc<WebRuntime>>()?))
    }
}

#[must_use]
pub(crate) fn route_set() -> RouteSet {
    let mut set = RouteSet::at::<WebState>(BasePath::Web(WebBase::Root), RouteGroup::Extensions)
        .service_with("/admin", |s: &WebState| s.0.admin(), SERVED)
        .service_with("/bridge-auth", |s: &WebState| s.0.bridge_auth(), SERVED)
        .service_with("/share", |s: &WebState| s.0.share.clone(), SERVED);
    for prefix in PUBLIC_API_PREFIXES {
        set = set.service_with(prefix, move |s: &WebState| s.0.public_api(prefix), SERVED);
    }
    debug_assert!(
        PUBLIC_API_PREFIXES
            .iter()
            .all(|p| p.starts_with(PUBLIC_API))
    );
    set.build()
}
