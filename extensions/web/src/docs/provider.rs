use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use systemprompt::template_provider::{PageContext, PageDataProvider};

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

        let cards: Vec<String> = children
            .iter()
            .filter_map(|child| {
                let title = child.get("title")?.as_str()?;
                let description = child.get("description")?.as_str()?;
                let url = child.get("url")?.as_str()?;

                Some(format!(
                    r#"<a href="{}" class="docs-card">
  <h3 class="docs-card-title">{}</h3>
  <p class="docs-card-description">{}</p>
</a>"#,
                    url,
                    html_escape(title),
                    html_escape(description)
                ))
            })
            .collect();

        if cards.is_empty() {
            None
        } else {
            Some(cards.join("\n"))
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
            "docs-page".to_string(),
            "guide".to_string(),
            "reference".to_string(),
            "tutorial".to_string(),
            "docs".to_string(),
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
