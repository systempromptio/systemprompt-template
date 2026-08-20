//! JSON API, webhook, and secrets routes.

use std::sync::Arc;

use axum::Router;
use axum::routing::post;

use systemprompt::oauth::SessionCreationService;

use super::pools::DbHandles;
use crate::extension::WebExtension;
use crate::{admin, api};

pub(crate) fn build(
    db: &DbHandles,
    session_service: &Arc<SessionCreationService>,
    sf_deps: admin::SalesforceDeps,
) -> Router {
    let admin_api = admin::admin_router(Arc::clone(&db.read), &db.write);
    let webhook_api =
        admin::hooks_webhook_router(Arc::clone(&db.write), Arc::clone(session_service));
    let secrets_api = admin::secrets_router(Arc::clone(&db.write));
    let links_router = api::router(Arc::clone(&db.read), WebExtension::blog_config());

    Router::new()
        .route(
            "/auth/session",
            post(api::auth::set_session).delete(api::auth::clear_session),
        )
        .merge(links_router)
        .merge(webhook_api)
        .merge(secrets_api)
        .merge(admin::salesforce_api_router(sf_deps))
        .nest("/admin", admin_api)
}

pub(crate) fn share(db: &DbHandles) -> Router {
    admin::share_manifest_router(Arc::clone(&db.read))
}
