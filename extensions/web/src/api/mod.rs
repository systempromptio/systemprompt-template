pub mod handlers;
mod types;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

use crate::config::BlogConfigValidated;

pub use types::*;

pub fn router(pool: Arc<PgPool>, config: Option<Arc<BlogConfigValidated>>) -> Router {
    let state = BlogState { pool, config };

    Router::new()
        .route("/links/generate", post(handlers::generate_link_handler))
        .route("/links/click", post(handlers::record_click_handler))
        .route("/links", get(handlers::list_links_handler))
        .route(
            "/links/{link_id}/performance",
            get(handlers::link_performance_handler),
        )
        .route(
            "/links/{link_id}/clicks",
            get(handlers::link_clicks_handler),
        )
        .route(
            "/links/campaigns/{campaign_id}/performance",
            get(handlers::campaign_performance_handler),
        )
        .route("/links/journey", get(handlers::content_journey_handler))
        .route("/engagement", post(handlers::engagement_handler))
        .route(
            "/engagement/batch",
            post(handlers::engagement_batch_handler),
        )
        .route("/search", get(handlers::search_handler))
        .with_state(state)
}

pub fn redirect_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/{short_code}", get(handlers::redirect_handler))
        .with_state(pool)
}

#[derive(Clone)]
pub struct BlogState {
    pub pool: Arc<PgPool>,
    pub config: Option<Arc<BlogConfigValidated>>,
}
