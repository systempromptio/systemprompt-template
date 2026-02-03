use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use systemprompt::database::Database;
use systemprompt::template_provider::{PageContext, PageDataProvider};

use super::renderers::render_blog_cards;
use super::types::BlogPost;

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

    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
        let Some(db) = ctx.db_pool::<Arc<Database>>() else {
            tracing::warn!("BlogListPageDataProvider: No database in context");
            return Ok(json!({ "POSTS": "" }));
        };

        let Some(pool) = db.pool() else {
            tracing::warn!("BlogListPageDataProvider: Pool not initialized");
            return Ok(json!({ "POSTS": "" }));
        };

        let posts = sqlx::query_as!(
            BlogPost,
            r#"
            SELECT
                slug,
                title,
                description,
                image,
                category,
                published_at
            FROM markdown_content
            WHERE source_id = 'blog'
            ORDER BY published_at DESC
            "#
        )
        .fetch_all(&*pool)
        .await?;

        tracing::debug!(count = posts.len(), "Fetched blog posts");

        let cards_html = render_blog_cards(&posts);

        Ok(json!({
            "POSTS": cards_html
        }))
    }
}
