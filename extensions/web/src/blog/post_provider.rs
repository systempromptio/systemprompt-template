use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use systemprompt::database::Database;
use systemprompt::models::Config;
use systemprompt::template_provider::{PageContext, PageDataProvider};

use super::renderers::{render_references, render_related_posts, render_social_action_bar};
use super::types::RelatedPost;

pub struct BlogPostPageDataProvider;

impl BlogPostPageDataProvider {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for BlogPostPageDataProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PageDataProvider for BlogPostPageDataProvider {
    fn provider_id(&self) -> &'static str {
        "blog-post-metadata"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec!["blog".to_string()]
    }

    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
        let item = ctx
            .content_item()
            .ok_or_else(|| anyhow::anyhow!("Content item required for blog post"))?;

        let mut data = json!({});

        if let Some(obj) = data.as_object_mut() {
            if let Some(title) = item.get("title").and_then(|v| v.as_str()) {
                obj.insert("TITLE".to_string(), Value::String(title.to_string()));
            }
            if let Some(desc) = item.get("description").and_then(|v| v.as_str()) {
                obj.insert("DESCRIPTION".to_string(), Value::String(desc.to_string()));
            }
            if let Some(author) = item.get("author").and_then(|v| v.as_str()) {
                obj.insert("AUTHOR".to_string(), Value::String(author.to_string()));
            }
            if let Some(keywords) = item.get("keywords").and_then(|v| v.as_str()) {
                obj.insert("KEYWORDS".to_string(), Value::String(keywords.to_string()));
            }

            if let Some(image) = item.get("image").and_then(|v| v.as_str()) {
                obj.insert(
                    "FEATURED_IMAGE".to_string(),
                    Value::String(image.to_string()),
                );
                obj.insert("IMAGE".to_string(), Value::String(image.to_string()));
            }

            if let Some(published) = item.get("published_at").and_then(|v| v.as_str()) {
                obj.insert("DATE_ISO".to_string(), Value::String(published.to_string()));
                obj.insert(
                    "DATE_PUBLISHED".to_string(),
                    Value::String(published.to_string()),
                );
                if let Ok(dt) = DateTime::parse_from_rfc3339(published) {
                    obj.insert(
                        "DATE".to_string(),
                        Value::String(dt.format("%B %d, %Y").to_string()),
                    );
                } else if let Ok(dt) = published.parse::<DateTime<Utc>>() {
                    obj.insert(
                        "DATE".to_string(),
                        Value::String(dt.format("%B %d, %Y").to_string()),
                    );
                }
            }
            if let Some(updated) = item.get("updated_at").and_then(|v| v.as_str()) {
                obj.insert(
                    "DATE_MODIFIED_ISO".to_string(),
                    Value::String(updated.to_string()),
                );
            }

            if let Some(content) = item.get("content").and_then(|v| v.as_str()) {
                let word_count = content.split_whitespace().count();
                let read_time = (word_count / 200).max(1);
                obj.insert(
                    "READ_TIME".to_string(),
                    Value::String(read_time.to_string()),
                );
            }

            let slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");
            let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");

            let org_url =
                Config::get().map_or_else(|_| String::new(), |c| c.api_external_url.clone());
            let social_bar = render_social_action_bar(slug, title, &org_url);
            obj.insert("SOCIAL_ACTION_BAR".to_string(), Value::String(social_bar));

            if let Some(links) = item.get("links") {
                if let Some(refs_html) = render_references(links) {
                    obj.insert("REFERENCES".to_string(), Value::String(refs_html));
                }
            }

            if let Some(db) = ctx.db_pool::<Arc<Database>>() {
                if let Some(pool) = db.pool() {
                    let related = sqlx::query_as!(
                        RelatedPost,
                        r#"
                        SELECT slug, title
                        FROM markdown_content
                        WHERE source_id = 'blog'
                        AND slug != $1
                        ORDER BY published_at DESC
                        LIMIT 3
                        "#,
                        slug
                    )
                    .fetch_all(&*pool)
                    .await
                    .inspect_err(|e| tracing::warn!(error = %e, "Failed to fetch related posts"))
                    .unwrap_or_else(|_| Vec::new());

                    if let Some(related_html) = render_related_posts(&related) {
                        obj.insert("SOCIAL_CONTENT".to_string(), Value::String(related_html));
                    }
                }
            }
        }

        Ok(data)
    }

    fn priority(&self) -> u32 {
        60
    }
}
