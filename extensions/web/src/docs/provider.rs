use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use systemprompt::extension::prelude::*;

use super::types::DocsLearningContent;
use crate::utils::html_escape;

pub struct DocsPageDataProvider;

impl DocsPageDataProvider {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn render_children_cards(children: &[Value]) -> Option<String> {
        if children.is_empty() {
            return None;
        }

        // Pre-allocate with estimated capacity to avoid intermediate Vec<String> + join
        let mut result = String::with_capacity(children.len() * 128);
        let mut first = true;

        for child in children {
            let Some(title) = child.get("title").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(description) = child.get("description").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(url) = child.get("url").and_then(|v| v.as_str()) else {
                continue;
            };

            if !first {
                result.push('\n');
            }
            first = false;

            use std::fmt::Write;
            let _ = write!(
                result,
                r#"<a href="{}" class="docs-card">
  <h3 class="docs-card-title">{}</h3>
  <p class="docs-card-description">{}</p>
</a>"#,
                html_escape(url),
                html_escape(title),
                html_escape(description)
            );
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
}

impl Default for DocsPageDataProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PageDataProvider for DocsPageDataProvider {
    fn provider_id(&self) -> &'static str {
        "docs-metadata"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec![
            "docs-page".into(),
            "guide".into(),
            "reference".into(),
            "tutorial".into(),
            "docs".into(),
        ]
    }

    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
        let item = ctx
            .content_item()
            .ok_or_else(|| anyhow::anyhow!("Content item required for docs page"))?;

        let learning_content = DocsLearningContent::from_content_item(item);
        let mut data = learning_content.to_template_data();

        if let Some(obj) = data.as_object_mut() {
            if let Some(title) = item.get("title").and_then(|v| v.as_str()) {
                obj.insert("TITLE".to_string(), Value::String(title.to_string()));
            }
            if let Some(desc) = item.get("description").and_then(|v| v.as_str()) {
                obj.insert("DESCRIPTION".to_string(), Value::String(desc.to_string()));
            }
            if let Some(slug) = item.get("slug").and_then(|v| v.as_str()) {
                obj.insert("SLUG".to_string(), Value::String(slug.to_string()));
            }
            if let Some(author) = item.get("author").and_then(|v| v.as_str()) {
                obj.insert("AUTHOR".to_string(), Value::String(author.to_string()));
            }
            if let Some(keywords) = item.get("keywords").and_then(|v| v.as_str()) {
                obj.insert("KEYWORDS".to_string(), Value::String(keywords.to_string()));
            }
            if let Some(image) = item.get("image").and_then(|v| v.as_str()) {
                obj.insert("IMAGE".to_string(), Value::String(image.to_string()));
            }

            if let Some(updated) = item.get("updated_at").and_then(|v| v.as_str()) {
                obj.insert(
                    "DATE_MODIFIED_ISO".to_string(),
                    Value::String(updated.to_string()),
                );
                if let Ok(dt) = DateTime::parse_from_rfc3339(updated) {
                    obj.insert(
                        "DATE_MODIFIED".to_string(),
                        Value::String(dt.format("%B %d, %Y").to_string()),
                    );
                } else if let Ok(dt) = updated.parse::<DateTime<Utc>>() {
                    obj.insert(
                        "DATE_MODIFIED".to_string(),
                        Value::String(dt.format("%B %d, %Y").to_string()),
                    );
                }
            }

            if let Some(published) = item.get("published_at").and_then(|v| v.as_str()) {
                obj.insert("DATE_ISO".to_string(), Value::String(published.to_string()));
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
        }

        if let Some(children) = item.get("children").and_then(|v| v.as_array()) {
            if let Some(children_html) = Self::render_children_cards(children) {
                if let Some(obj) = data.as_object_mut() {
                    obj.insert("CHILDREN".to_string(), Value::String(children_html));
                }
            }
        }

        Ok(data)
    }

    fn priority(&self) -> u32 {
        60
    }
}
