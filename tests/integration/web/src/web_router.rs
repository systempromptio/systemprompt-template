//! The web extension's HTTP router, assembled from a live pool.
//!
//! `Extension::router` is the only public way in: it takes the type-erased
//! `ExtensionContext`, so the stub below hands it a real `Database` built on a
//! throwaway pool. That exercises the whole assembly — pool extraction, the
//! session service the Salesforce and webhook planes need, and the JSON/API
//! plane — and pins where each plane is mounted by driving requests through the
//! returned `axum::Router`.
//!
//! No profile is bootstrapped in a test process, so the admin template
//! directory cannot be resolved and the SSR nests are skipped. That is the
//! documented degradation: the API plane must still mount at its normal prefix,
//! which is what the last tests here assert.

use std::any::Any;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use sqlx::PgPool;
use systemprompt::database::Database;
use systemprompt::extension::prelude::{Extension, ExtensionContext};
use systemprompt::traits::{ConfigProvider, DatabaseHandle};
use systemprompt_web_extension::WebExtension;
use tower::ServiceExt as _;

use crate::tempdb::TempDb;

#[derive(Debug)]
struct StubConfig;

impl ConfigProvider for StubConfig {
    fn get(&self, _key: &str) -> Option<String> {
        None
    }
    fn database_url(&self) -> &str {
        "postgres://unused/unused"
    }
    fn system_path(&self) -> &str {
        "/tmp"
    }
    fn api_port(&self) -> u16 {
        0
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct PoolCtx {
    database: Arc<Database>,
}

impl PoolCtx {
    fn new(pool: &Arc<PgPool>) -> Self {
        Self {
            database: Arc::new(Database::from_pools(
                Arc::clone(pool),
                Some(Arc::clone(pool)),
            )),
        }
    }
}

impl ExtensionContext for PoolCtx {
    fn config(&self) -> Arc<dyn ConfigProvider> {
        Arc::new(StubConfig)
    }
    fn database(&self) -> Arc<dyn DatabaseHandle> {
        Arc::clone(&self.database) as Arc<dyn DatabaseHandle>
    }
    fn get_extension(&self, _id: &str) -> Option<Arc<dyn Extension>> {
        None
    }
}

/// A context whose database handle is not a core `Database`, which is the one
/// shape pool extraction cannot downcast.
struct ForeignDbCtx;

#[derive(Debug)]
struct ForeignDb;

impl DatabaseHandle for ForeignDb {
    fn is_connected(&self) -> bool {
        true
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ExtensionContext for ForeignDbCtx {
    fn config(&self) -> Arc<dyn ConfigProvider> {
        Arc::new(StubConfig)
    }
    fn database(&self) -> Arc<dyn DatabaseHandle> {
        Arc::new(ForeignDb)
    }
    fn get_extension(&self, _id: &str) -> Option<Arc<dyn Extension>> {
        None
    }
}

async fn status_of(router: axum::Router, method: &str, path: &str) -> StatusCode {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .body(Body::empty())
        .expect("build the probe request");
    router
        .oneshot(request)
        .await
        .expect("the router is infallible")
        .status()
}

#[tokio::test]
async fn the_router_assembles_from_a_context_carrying_a_real_database() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let built = WebExtension::new().router(&PoolCtx::new(&db.pool));

    let mounted = built.expect("a context with a real database yields a router");
    assert_eq!(
        mounted.base_path, "/",
        "the extension owns the site root, not a sub-prefix"
    );
    assert!(
        !mounted.requires_auth,
        "the plane mounts public; per-route auth is the site-auth config's job"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_context_without_a_core_database_yields_no_router() {
    assert!(
        WebExtension::new().router(&ForeignDbCtx).is_none(),
        "pool extraction fails closed rather than mounting routes with no database"
    );
}

#[tokio::test]
async fn the_session_route_is_mounted_under_the_public_api_prefix() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let router = WebExtension::new()
        .router(&PoolCtx::new(&db.pool))
        .expect("router builds")
        .router;

    let status = status_of(router, "GET", "/api/public/auth/session").await;

    assert_eq!(
        status,
        StatusCode::METHOD_NOT_ALLOWED,
        "the path exists and serves POST/DELETE only, so GET is 405 rather than 404"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_admin_json_api_is_nested_below_the_public_api_prefix() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let router = WebExtension::new()
        .router(&PoolCtx::new(&db.pool))
        .expect("router builds")
        .router;

    let status = status_of(router, "GET", "/api/public/admin/no-such-endpoint").await;

    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "an unknown path under the admin nest is a 404 from the admin router"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_path_outside_every_mounted_plane_is_not_found() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let router = WebExtension::new()
        .router(&PoolCtx::new(&db.pool))
        .expect("router builds")
        .router;

    let status = status_of(router, "GET", "/definitely-not-a-web-extension-route").await;

    assert_eq!(status, StatusCode::NOT_FOUND);

    db.cleanup().await;
}

// The SSR nests need the admin template directory, which is resolved from the
// bootstrapped profile; a test process has none. The documented degradation is
// that assembly still succeeds and only the SSR nests are skipped.
#[tokio::test]
async fn the_ssr_dashboard_is_skipped_when_the_profile_cannot_be_resolved() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let router = WebExtension::new()
        .router(&PoolCtx::new(&db.pool))
        .expect("the API plane mounts even with no template engine")
        .router;

    let status = status_of(router, "GET", "/admin/login").await;

    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "with no template engine the dashboard is absent rather than erroring"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn two_builds_from_the_same_pool_mount_the_same_surface() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let first = WebExtension::new()
        .router(&PoolCtx::new(&db.pool))
        .expect("router builds");
    let second = WebExtension::new()
        .router(&PoolCtx::new(&db.pool))
        .expect("router builds again from an independent context");

    assert_eq!(first.base_path, second.base_path);
    assert_eq!(
        status_of(first.router, "GET", "/api/public/auth/session").await,
        status_of(second.router, "GET", "/api/public/auth/session").await,
        "router assembly is a pure function of the context"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_extension_advertises_the_site_auth_contract_the_router_depends_on() {
    let auth = WebExtension::new()
        .site_auth()
        .expect("the web extension protects the dashboard");

    assert_eq!(auth.login_path, "/admin/login");
    assert!(
        auth.protected_prefixes.contains(&"/admin"),
        "the SSR dashboard prefix is protected"
    );
    assert!(
        auth.public_prefixes.contains(&"/admin/login"),
        "the login page itself is exempt, or sign-in would be unreachable"
    );
}
