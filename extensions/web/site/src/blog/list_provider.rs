use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};
use systemprompt::database::Database;
use systemprompt::extension::prelude::*;

use super::renderers::render_blog_cards;
use crate::repositories::blog::list_blog_posts;
use systemprompt_web_shared::error::BlogError;

#[derive(Debug, Clone, Copy)]
pub struct BlogListPageDataProvider;

impl BlogListPageDataProvider {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for BlogListPageDataProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PageDataProvider for BlogListPageDataProvider {
    fn provider_id(&self) -> &'static str {
        "blog-list-posts"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec!["blog-list".to_string()]
    }

    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value, systemprompt::traits::ProviderError> {
        let Some(db) = ctx.db_pool::<Arc<Database>>() else {
            tracing::warn!("BlogListPageDataProvider: No database in context");
            return Ok(json!({ "POSTS": "" }));
        };

        let Some(pool) = db.pool() else {
            tracing::warn!("BlogListPageDataProvider: Pool not initialized");
            return Ok(json!({ "POSTS": "" }));
        };

        let posts = list_blog_posts(&pool)
            .await
            .map_err(BlogError::Database)
            .map_err(|e| systemprompt::traits::ProviderError::from(anyhow::Error::from(e)))?;

        tracing::debug!(count = posts.len(), "Fetched blog posts");

        let cards_html = render_blog_cards(&posts);

        Ok(json!({
            "POSTS": cards_html
        }))
    }
}
