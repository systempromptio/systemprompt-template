use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use systemprompt::template_provider::{PageContext, PageDataProvider};

use crate::html_escape;
use crate::utils::playbook_categories::{
    BUILD, BUILD_PREFIX, CLI, CLI_PREFIX, CONTENT, CONTENT_PREFIX, GUIDE, GUIDE_PREFIX, VALIDATION,
    VALIDATION_PREFIX,
};

pub struct PlaybookPageDataProvider;

impl PlaybookPageDataProvider {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    fn derive_category(slug: &str) -> &'static str {
        if slug.starts_with(GUIDE_PREFIX) {
            GUIDE
        } else if slug.starts_with(CLI_PREFIX) {
            CLI
        } else if slug.starts_with(BUILD_PREFIX) {
            BUILD
        } else if slug.starts_with(CONTENT_PREFIX) {
            CONTENT
        } else if slug.starts_with(VALIDATION_PREFIX) {
            VALIDATION
        } else {
            ""
        }
    }

    #[must_use]
    pub fn render_related_playbooks(playbooks: &[Value]) -> Option<String> {
        if playbooks.is_empty() {
            return None;
        }

        let links: Vec<String> = playbooks
            .iter()
            .filter_map(|pb| {
                let title = pb.get("title")?.as_str()?;
                let url = pb.get("url")?.as_str()?;

                Some(format!(
                    r#"<a href="{}" class="playbook-related-card">
  <span class="playbook-related-card-title">{}</span>
</a>"#,
                    html_escape(url),
                    html_escape(title)
                ))
            })
            .collect();

        if links.is_empty() {
            None
        } else {
            Some(links.join("\n"))
        }
    }

    #[must_use]
    pub fn render_same_category_playbooks(playbooks: &[Value]) -> Option<String> {
        if playbooks.is_empty() {
            return None;
        }

        let links: Vec<String> = playbooks
            .iter()
            .filter_map(|pb| {
                let title = pb.get("title")?.as_str()?;
                let url = pb.get("url")?.as_str()?;
                let description = pb.get("description").and_then(|v| v.as_str()).unwrap_or("");

                Some(format!(
                    r#"<a href="{}" class="playbook-category-link">
  <span class="playbook-category-link-title">{}</span>
  <span class="playbook-category-link-desc">{}</span>
</a>"#,
                    html_escape(url),
                    html_escape(title),
                    html_escape(description)
                ))
            })
            .collect();

        if links.is_empty() {
            None
        } else {
            Some(links.join("\n"))
        }
    }
}

impl Default for PlaybookPageDataProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PageDataProvider for PlaybookPageDataProvider {
    fn provider_id(&self) -> &'static str {
        "playbook-page-metadata"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec!["playbook".to_string()]
    }

    #[allow(clippy::too_many_lines)]
    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
        let item = ctx
            .content_item()
            .ok_or_else(|| anyhow::anyhow!("Content item required for playbook page"))?;

        let mut data = serde_json::Map::new();

        if let Some(title) = item.get("title").and_then(|v| v.as_str()) {
            data.insert("TITLE".to_string(), Value::String(title.to_string()));
        }
        if let Some(desc) = item.get("description").and_then(|v| v.as_str()) {
            data.insert("DESCRIPTION".to_string(), Value::String(desc.to_string()));
        }
        if let Some(slug) = item.get("slug").and_then(|v| v.as_str()) {
            data.insert("SLUG".to_string(), Value::String(slug.to_string()));

            let category = Self::derive_category(slug);
            if !category.is_empty() {
                data.insert("CATEGORY".to_string(), Value::String(category.to_string()));
            }
        }
        if let Some(author) = item.get("author").and_then(|v| v.as_str()) {
            data.insert("AUTHOR".to_string(), Value::String(author.to_string()));
        }

        if let Some(keywords) = item.get("keywords") {
            if let Some(kw_str) = keywords.as_str() {
                data.insert("KEYWORDS".to_string(), Value::String(kw_str.to_string()));
                let keywords_array: Vec<Value> = kw_str
                    .split(',')
                    .map(|s| Value::String(s.trim().to_string()))
                    .collect();
                data.insert("KEYWORDS_ARRAY".to_string(), Value::Array(keywords_array));
            } else if let Some(kw_arr) = keywords.as_array() {
                let keywords_str = kw_arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                data.insert("KEYWORDS".to_string(), Value::String(keywords_str));
                data.insert("KEYWORDS_ARRAY".to_string(), keywords.clone());
            }
        }

        if let Some(published) = item
            .get("published_at")
            .or_else(|| item.get("date"))
            .and_then(|v| v.as_str())
        {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(published) {
                data.insert(
                    "DATE".to_string(),
                    Value::String(dt.format("%B %d, %Y").to_string()),
                );
                data.insert(
                    "DATE_ISO".to_string(),
                    Value::String(dt.format("%Y-%m-%d").to_string()),
                );
            } else {
                data.insert("DATE".to_string(), Value::String(published.to_string()));
                data.insert("DATE_ISO".to_string(), Value::String(published.to_string()));
            }
        }

        if let Some(priority) = item.get("priority") {
            if let Some(p_str) = priority.as_str() {
                data.insert("PRIORITY".to_string(), Value::String(p_str.to_string()));
            } else if let Some(p_num) = priority.as_i64() {
                data.insert("PRIORITY".to_string(), Value::String(p_num.to_string()));
            }
        }

        if let Some(related) = item.get("related_playbooks").and_then(|v| v.as_array()) {
            if !related.is_empty() {
                data.insert(
                    "RELATED_PLAYBOOKS".to_string(),
                    Value::Array(related.clone()),
                );
                if let Some(html) = Self::render_related_playbooks(related) {
                    data.insert("RELATED_PLAYBOOKS_HTML".to_string(), Value::String(html));
                }
            }
        }

        if let Some(same_cat) = item
            .get("same_category_playbooks")
            .and_then(|v| v.as_array())
        {
            if !same_cat.is_empty() {
                data.insert(
                    "SAME_CATEGORY_PLAYBOOKS".to_string(),
                    Value::Array(same_cat.clone()),
                );
                if let Some(html) = Self::render_same_category_playbooks(same_cat) {
                    data.insert(
                        "SAME_CATEGORY_PLAYBOOKS_HTML".to_string(),
                        Value::String(html),
                    );
                }
            }
        }

        if let Some(related_code) = item.get("related_code").and_then(|v| v.as_array()) {
            if !related_code.is_empty() {
                data.insert(
                    "RELATED_CODE".to_string(),
                    Value::Array(related_code.clone()),
                );
            }
        }

        Ok(Value::Object(data))
    }

    fn priority(&self) -> u32 {
        60
    }
}
