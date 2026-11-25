//! JSON API, webhook, and secrets routes.

use std::sync::Arc;

use axum::Router;
use axum::routing::post;

use systemprompt::governance::policy::RateWindowRepository;
use systemprompt::oauth::SessionCreationService;

use super::pools::DbHandles;
use crate::extension::WebExtension;
use crate::{admin, api};

pub(crate) fn build(
    db: &DbHandles,
    session_service: &Arc<SessionCreationService>,
    settings: &crate::WebSettings,
) -> Router {
    let admin_api = admin::admin_router(&db.db, Arc::clone(&db.read), &db.write, db.owner.clone());
    let webhook_api = admin::hooks_webhook_router(
        Arc::clone(&db.write),
        Arc::clone(session_service),
        RateWindowRepository::new(db.write.as_ref().clone()),
    );
    let secrets_api = admin::secrets_router(Arc::clone(&db.write));
    let bridge_identity = admin::bridge_identity_router(Arc::clone(&db.read));
    let links_router = api::router(Arc::clone(&db.read), WebExtension::blog_config(settings));

    Router::new()
        .route(
            "/auth/session",
            post(api::auth::set_session)
                .delete(api::auth::clear_session)
                .with_state(api::auth::CookieSecure(settings.use_https)),
        )
        .merge(links_router)
        .merge(bridge_identity)
        .merge(admin::connector_api_router(Arc::clone(&db.write)))
        .merge(webhook_api)
        .merge(secrets_api)
        .nest("/admin", admin_api)
}

pub(crate) fn share(db: &DbHandles) -> Router {
    admin::share_manifest_router(Arc::clone(&db.read))
}
