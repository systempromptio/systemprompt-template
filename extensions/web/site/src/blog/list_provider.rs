//! Page data provider for the blog index route.

use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;
use systemprompt::database::Database;
use systemprompt::extension::prelude::*;

use super::renderers::render_blog_cards;
use crate::repositories::blog::list_blog_posts;
use systemprompt_web_shared::error::BlogError;

/// Template context for the blog list page (`blog-list.html`).
///
/// The template consumes a single `{{{POSTS}}}` triple-mustache holding the
/// pre-rendered blog-card HTML.
#[derive(Debug, Serialize)]
struct BlogListContext {
    #[serde(rename = "POSTS")]
    posts: String,
}

impl BlogListContext {
    fn to_value(&self) -> Result<Value, systemprompt::traits::ProviderError> {
        serde_json::to_value(self)
            .map_err(|e| systemprompt::traits::ProviderError::Internal(e.to_string()))
    }
}

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
        vec!["blog-list".to_owned()]
    }

    async fn provide_page_data(
        &self,
        ctx: &PageContext<'_>,
    ) -> Result<Value, systemprompt::traits::ProviderError> {
        let Some(db) = ctx.db_pool::<Arc<Database>>() else {
            tracing::warn!("BlogListPageDataProvider: No database in context");
            return BlogListContext {
                posts: String::new(),
            }
            .to_value();
        };

        let Some(pool) = db.pool() else {
            tracing::warn!("BlogListPageDataProvider: Pool not initialized");
            return BlogListContext {
                posts: String::new(),
            }
            .to_value();
        };

        let posts = list_blog_posts(&pool)
            .await
            .map_err(BlogError::Database)
            .map_err(|e| systemprompt::traits::ProviderError::Internal(e.to_string()))?;

        tracing::debug!(count = posts.len(), "Fetched blog posts");

        let cards_html = render_blog_cards(&posts);

        BlogListContext { posts: cards_html }.to_value()
    }
}
