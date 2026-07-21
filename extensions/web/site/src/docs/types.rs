//! Template context types for `docs-page.html`.
//!
//! Falsey flags are omitted from the rendered map so the template's `{{#if}}`
//! guards behave, rather than seeing a present-but-false key.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

fn parse_field<T: DeserializeOwned + Default>(item: &Value, field: &str) -> T {
    let Some(v) = item.get(field) else {
        return T::default();
    };
    match serde_json::from_value(v.clone()) {
        Ok(parsed) => parsed,
        Err(e) => {
            tracing::warn!(field, error = %e, "Parse failed");
            T::default()
        },
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DocsLearningContent {
    pub after_reading_this: Vec<String>,
    pub related_playbooks: Vec<RelatedLink>,
    pub related_code: Vec<RelatedLink>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelatedLink {
    pub title: String,
    pub url: String,
}

/// Template context fragment for the docs-page learning aside. Empty lists and
/// a `false` flag are omitted so the `{{#if}}` guards in `docs-page.html`
/// behave exactly as when the keys were inserted conditionally.
#[derive(Debug, Serialize)]
struct DocsLearningTemplateData {
    #[serde(rename = "AFTER_READING_THIS", skip_serializing_if = "Vec::is_empty")]
    after_reading_this: Vec<String>,
    #[serde(rename = "RELATED_PLAYBOOKS", skip_serializing_if = "Vec::is_empty")]
    related_playbooks: Vec<RelatedLink>,
    #[serde(rename = "RELATED_CODE", skip_serializing_if = "Vec::is_empty")]
    related_code: Vec<RelatedLink>,
    #[serde(rename = "HAS_LEARNING_CONTENT", skip_serializing_if = "is_false")]
    has_learning_content: bool,
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde skip_serializing_if requires a &T predicate signature"
)]
const fn is_false(value: &bool) -> bool {
    !*value
}

impl DocsLearningContent {
    #[must_use]
    pub const fn has_content(&self) -> bool {
        !self.after_reading_this.is_empty()
            || !self.related_playbooks.is_empty()
            || !self.related_code.is_empty()
    }

    #[must_use]
    pub fn from_content_item(item: &Value) -> Self {
        Self {
            after_reading_this: parse_field(item, "after_reading_this"),
            related_playbooks: parse_field(item, "related_playbooks"),
            related_code: parse_field(item, "related_code"),
        }
    }

    #[must_use]
    pub fn to_template_data(&self) -> Value {
        let data = DocsLearningTemplateData {
            after_reading_this: self.after_reading_this.clone(),
            related_playbooks: self.related_playbooks.clone(),
            related_code: self.related_code.clone(),
            has_learning_content: self.has_content(),
        };
        serde_json::to_value(data).unwrap_or_else(|_| Value::Object(serde_json::Map::new()))
    }
}
