//! JSON API, webhook, and secrets routes.

use std::sync::Arc;

use axum::Router;
use axum::routing::post;

use systemprompt::oauth::SessionCreationService;

use super::pools::DbHandles;
use crate::extension::WebExtension;
use crate::{admin, api};

pub(crate) fn build(db: &DbHandles, session_service: &Arc<SessionCreationService>) -> Router {
    let admin_api = admin::admin_router_with_pools(Arc::clone(&db.read), &db.write);
    let webhook_api =
        admin::hooks_webhook_router(Arc::clone(&db.write), Arc::clone(session_service));
    let secrets_api = admin::secrets_router(Arc::clone(&db.write));
    let links_router = api::router(Arc::clone(&db.read), WebExtension::blog_config());

    let salesforce_api = admin::salesforce_api_router(admin::SalesforceDeps {
        config: WebExtension::salesforce_config()
            .unwrap_or_else(|| Arc::new(admin::SalesforceConfig::disabled())),
        write_pool: Arc::clone(&db.write),
    });

    Router::new()
        .route(
            "/auth/session",
            post(api::auth::set_session).delete(api::auth::clear_session),
        )
        .merge(links_router)
        .merge(admin::bridge_identity_router(Arc::clone(&db.read)))
        .merge(salesforce_api)
        .merge(admin::connector_api_router(Arc::clone(&db.write)))
        .merge(webhook_api)
        .merge(secrets_api)
        .nest("/admin", admin_api)
}

pub(crate) fn share(db: &DbHandles) -> Router {
    admin::share_manifest_router(Arc::clone(&db.read))
}
