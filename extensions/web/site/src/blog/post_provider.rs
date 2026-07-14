use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use systemprompt::database::Database;
use systemprompt::extension::prelude::*;
use systemprompt::models::Config;

use super::renderers::{render_references, render_related_posts, render_social_action_bar};
use crate::repositories::blog::list_related_posts;
use systemprompt_web_shared::error::BlogError;

/// Template context for a rendered blog post (`blog-post.html`).
///
/// Every field is optional and omitted when absent so the template's `{{KEY}}`
/// and `{{#if}}` guards behave exactly as when the keys were inserted
/// conditionally into a map.
#[derive(Debug, Default, Serialize)]
struct BlogPostContext {
    #[serde(rename = "TITLE", skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(rename = "DESCRIPTION", skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(rename = "AUTHOR", skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(rename = "KEYWORDS", skip_serializing_if = "Option::is_none")]
    keywords: Option<String>,
    #[serde(rename = "FEATURED_IMAGE", skip_serializing_if = "Option::is_none")]
    featured_image: Option<String>,
    #[serde(rename = "IMAGE", skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    #[serde(rename = "DATE_ISO", skip_serializing_if = "Option::is_none")]
    date_iso: Option<String>,
    #[serde(rename = "DATE_PUBLISHED", skip_serializing_if = "Option::is_none")]
    date_published: Option<String>,
    #[serde(rename = "DATE", skip_serializing_if = "Option::is_none")]
    date: Option<String>,
    #[serde(rename = "DATE_MODIFIED_ISO", skip_serializing_if = "Option::is_none")]
    date_modified_iso: Option<String>,
    #[serde(rename = "READ_TIME", skip_serializing_if = "Option::is_none")]
    read_time: Option<String>,
    #[serde(rename = "SOCIAL_ACTION_BAR", skip_serializing_if = "Option::is_none")]
    social_action_bar: Option<String>,
    #[serde(rename = "REFERENCES", skip_serializing_if = "Option::is_none")]
    references: Option<String>,
    #[serde(rename = "SOCIAL_CONTENT", skip_serializing_if = "Option::is_none")]
    social_content: Option<String>,
}

impl BlogPostContext {
    fn apply_basic_metadata(&mut self, item: &Value) {
        self.title = item.get("title").and_then(str_field);
        self.description = item.get("description").and_then(str_field);
        self.author = item.get("author").and_then(str_field);
        self.keywords = item.get("keywords").and_then(str_field);
        if let Some(image) = item.get("image").and_then(str_field) {
            self.featured_image = Some(image.clone());
            self.image = Some(image);
        }
    }

    fn apply_date_metadata(&mut self, item: &Value) {
        if let Some(published) = item.get("published_at").and_then(str_field) {
            self.date_iso = Some(published.clone());
            self.date_published = Some(published.clone());
            self.date = format_date(&published);
        }
        self.date_modified_iso = item.get("updated_at").and_then(str_field);
    }

    fn apply_read_time(&mut self, item: &Value) {
        if let Some(content) = item.get("content").and_then(|v| v.as_str()) {
            let word_count = content.split_whitespace().count();
            let read_time = (word_count / 200).max(1);
            self.read_time = Some(read_time.to_string());
        }
    }

    fn apply_social_and_references(&mut self, item: &Value) {
        let slug = item
            .get("slug")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let title = item
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        let org_url = Config::get().map_or_else(|_| String::new(), |c| c.api_external_url.clone());
        self.social_action_bar = Some(render_social_action_bar(slug, title, &org_url));

        if let Some(links) = item.get("links") {
            self.references = render_references(links);
        }
    }
}

fn str_field(value: &Value) -> Option<String> {
    value.as_str().map(str::to_owned)
}

fn format_date(published: &str) -> Option<String> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(published) {
        return Some(dt.format("%B %d, %Y").to_string());
    }
    published
        .parse::<DateTime<Utc>>()
        .ok()
        .map(|dt| dt.format("%B %d, %Y").to_string())
}

#[derive(Debug, Clone, Copy)]
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
        vec!["blog".to_owned()]
    }

    async fn provide_page_data(
        &self,
        ctx: &PageContext<'_>,
    ) -> Result<Value, systemprompt::traits::ProviderError> {
        let item = ctx
            .content_item()
            .ok_or_else(|| {
                BlogError::InvalidRequest("Content item required for blog post".to_owned())
            })
            .map_err(|e| systemprompt::traits::ProviderError::Internal(e.to_string()))?;

        let mut context = BlogPostContext::default();
        context.apply_basic_metadata(item);
        context.apply_date_metadata(item);
        context.apply_read_time(item);
        context.apply_social_and_references(item);

        let slug = item
            .get("slug")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        if let Some(db) = ctx.db_pool::<Arc<Database>>()
            && let Some(pool) = db.pool()
        {
            let related = list_related_posts(&pool, slug).await.unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to fetch related posts");
                Vec::new()
            });

            if let Some(related_html) = render_related_posts(&related) {
                context.social_content = Some(related_html);
            }
        }

        Ok(serde_json::to_value(context)?)
    }

    fn priority(&self) -> u32 {
        60
    }
}
