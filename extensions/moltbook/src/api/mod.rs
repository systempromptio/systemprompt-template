pub mod handlers;
mod types;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

pub use types::*;

use crate::client::MoltbookClient;

#[derive(Clone)]
pub struct MoltbookState {
    pub pool: Arc<PgPool>,
    pub clients: Arc<tokio::sync::RwLock<std::collections::HashMap<String, MoltbookClient>>>,
}

impl MoltbookState {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            pool,
            clients: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn get_client(&self, agent_id: &str) -> Option<MoltbookClient> {
        let clients = self.clients.read().await;
        clients.get(agent_id).cloned()
    }

    pub async fn register_client(
        &self,
        agent_id: String,
        api_key: String,
    ) -> Result<(), crate::error::MoltbookError> {
        let client = MoltbookClient::new(api_key)?;
        let mut clients = self.clients.write().await;
        clients.insert(agent_id, client);
        Ok(())
    }
}

pub fn router(pool: Arc<PgPool>) -> Router {
    let state = MoltbookState::new(pool);

    Router::new()
        .route("/health", get(handlers::health_handler))
        .route("/agents", get(handlers::list_agents_handler))
        .route("/agents/:agent_id", get(handlers::get_agent_handler))
        .route(
            "/agents/:agent_id/register",
            post(handlers::register_client_handler),
        )
        .route("/posts", post(handlers::create_post_handler))
        .route("/posts", get(handlers::list_posts_handler))
        .route("/posts/:post_id", get(handlers::get_post_handler))
        .route(
            "/posts/:post_id/comments",
            post(handlers::create_comment_handler),
        )
        .route(
            "/posts/:post_id/comments",
            get(handlers::list_comments_handler),
        )
        .route("/posts/:post_id/vote", post(handlers::vote_post_handler))
        .route("/feed", get(handlers::get_feed_handler))
        .route("/search/posts", get(handlers::search_posts_handler))
        .route("/search/submolts", get(handlers::search_submolts_handler))
        .route("/submolts/:name", get(handlers::get_submolt_handler))
        .with_state(state)
}
