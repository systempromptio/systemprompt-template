//! Runtime page data provider for documentation routes.

use std::fmt::Write;

use crate::format::format_date;
use async_trait::async_trait;
use serde::Serialize;
// JSON: content items and page data cross the provider trait as Values.
use serde_json::Value;
use systemprompt::extension::prelude::*;

use super::content_provider::ChildDoc;
use super::error::DocsError;
use super::types::{DocsLearningContent, DocsLearningTemplateData};
use systemprompt_web_shared::html_escape;

#[derive(Debug, Default, Serialize)]
struct DocsPageContext {
    #[serde(rename = "TITLE", skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(rename = "DESCRIPTION", skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(rename = "SLUG", skip_serializing_if = "Option::is_none")]
    slug: Option<String>,
    #[serde(rename = "AUTHOR", skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(rename = "KEYWORDS", skip_serializing_if = "Option::is_none")]
    keywords: Option<String>,
    #[serde(rename = "IMAGE", skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    #[serde(rename = "DATE_MODIFIED_ISO", skip_serializing_if = "Option::is_none")]
    date_modified_iso: Option<String>,
    #[serde(rename = "DATE_MODIFIED", skip_serializing_if = "Option::is_none")]
    date_modified: Option<String>,
    #[serde(rename = "DATE_ISO", skip_serializing_if = "Option::is_none")]
    date_iso: Option<String>,
    #[serde(rename = "DATE", skip_serializing_if = "Option::is_none")]
    date: Option<String>,
    #[serde(rename = "CHILDREN", skip_serializing_if = "Option::is_none")]
    children: Option<String>,
    #[serde(flatten)]
    learning: DocsLearningTemplateData,
}

// JSON: the helpers below read the content item the provider trait hands over
// as a Value; typed views are carved out field by field.
#[doc(hidden)]
pub fn str_field(item: &Value, field: &str) -> Option<String> {
    item.get(field).and_then(|v| v.as_str()).map(str::to_owned)
}

#[doc(hidden)]
pub fn parse_children(item: &Value) -> Vec<ChildDoc> {
    let Some(raw) = item.get("children") else {
        return Vec::new();
    };
    serde_json::from_value::<Vec<ChildDoc>>(raw.clone()).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to parse docs children");
        Vec::new()
    })
}

impl DocsPageContext {
    fn from_content_item(item: &Value) -> Self {
        let mut context = Self {
            title: str_field(item, "title"),
            description: str_field(item, "description"),
            slug: str_field(item, "slug"),
            author: str_field(item, "author"),
            keywords: str_field(item, "keywords"),
            image: str_field(item, "image"),
            ..Self::default()
        };

        if let Some(updated) = str_field(item, "updated_at") {
            context.date_modified = format_date(&updated);
            context.date_modified_iso = Some(updated);
        }
        if let Some(published) = str_field(item, "published_at") {
            context.date = format_date(&published);
            context.date_iso = Some(published);
        }

        context.learning = DocsLearningContent::from_content_item(item).template_data();
        context.children = DocsPageDataProvider::render_children_cards(&parse_children(item));
        context
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DocsPageDataProvider;

impl DocsPageDataProvider {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn render_children_cards(children: &[ChildDoc]) -> Option<String> {
        if children.is_empty() {
            return None;
        }

        let mut result = String::with_capacity(children.len() * 128);
        let mut first = true;

        for child in children {
            if !first {
                result.push('\n');
            }
            first = false;

            // Why: `fmt::Write` for `String` never returns `Err`; the result is
            // genuinely discardable.
            write!(
                result,
                r#"<a href="{}" class="docs-card">
  <h3 class="docs-card-title">{}</h3>
  <p class="docs-card-description">{}</p>
</a>"#,
                html_escape(&child.url),
                html_escape(&child.title),
                html_escape(&child.description)
            )
            .ok();
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

    async fn provide_page_data(
        &self,
        ctx: &PageContext<'_>,
        // JSON: required by trait contract
    ) -> Result<Value, systemprompt::traits::ProviderError> {
        let item = ctx
            .content_item()
            .ok_or(DocsError::ContentItemRequired)
            .map_err(|e| systemprompt::traits::ProviderError::Internal(e.to_string()))?;

        Ok(serde_json::to_value(DocsPageContext::from_content_item(
            item,
        ))?)
    }

    fn priority(&self) -> u32 {
        60
    }
}

systemprompt_web_shared::submit_page_data!(DocsPageDataProvider::new());
