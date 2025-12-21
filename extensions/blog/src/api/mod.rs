//! Blog API routes and handlers.

pub mod handlers;
mod types;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

use crate::BlogConfig;

pub use types::*;

/// Build the blog API router.
pub fn router(pool: Arc<PgPool>, config: BlogConfig) -> Router {
    let state = BlogState { pool, config };

    Router::new()
        // Content endpoints
        .route("/query", post(handlers::query_handler))
        .route("/:source_id", get(handlers::list_content_handler))
        .route("/:source_id/:slug", get(handlers::get_content_handler))
        // Link tracking endpoints
        .route("/links/generate", post(handlers::generate_link_handler))
        .route("/links", get(handlers::list_links_handler))
        .route(
            "/links/:link_id/performance",
            get(handlers::link_performance_handler),
        )
        .route("/links/:link_id/clicks", get(handlers::link_clicks_handler))
        .route(
            "/links/campaigns/:campaign_id/performance",
            get(handlers::campaign_performance_handler),
        )
        .route("/links/journey", get(handlers::content_journey_handler))
        .with_state(state)
}

/// Redirect router (mounted separately at /r/).
pub fn redirect_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/:short_code", get(handlers::redirect_handler))
        .with_state(pool)
}

/// State shared across all blog API handlers.
#[derive(Clone)]
pub struct BlogState {
    pub pool: Arc<PgPool>,
    pub config: BlogConfig,
}
