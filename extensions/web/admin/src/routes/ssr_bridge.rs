use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;
use tower_http::normalize_path::NormalizePathLayer;

use super::super::{handlers, middleware, templates::AdminTemplateEngine};

pub fn cowork_auth_ssr_router(pool: Arc<PgPool>, engine: AdminTemplateEngine) -> Router {
    let inner = Router::new()
        .route("/setup", get(handlers::ssr::cowork_setup_page))
        .route("/device-link", get(handlers::ssr::device_link_page))
        .route(
            "/device-link/approve",
            post(handlers::ssr::device_link_approve),
        )
        .route("/device-link/deny", post(handlers::ssr::device_link_deny))
        .layer(Extension(engine))
        .layer(axum_middleware::from_fn(
            middleware::marketplace_context_middleware,
        ))
        .layer(axum_middleware::from_fn(
            middleware::require_user_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            Arc::clone(&pool),
            middleware::user_context_middleware,
        ))
        .with_state(pool);

    Router::new().fallback_service(
        tower::ServiceBuilder::new()
            .layer(NormalizePathLayer::trim_trailing_slash())
            .service(inner),
    )
}
