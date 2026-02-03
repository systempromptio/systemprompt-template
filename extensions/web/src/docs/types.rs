use serde::{Deserialize, Serialize};
use serde_json::Value;

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

impl DocsLearningContent {
    #[must_use]
    pub fn has_content(&self) -> bool {
        !self.after_reading_this.is_empty()
            || !self.related_playbooks.is_empty()
            || !self.related_code.is_empty()
    }

    #[must_use]
    pub fn from_content_item(item: &Value) -> Self {
        Self {
            after_reading_this: item
                .get("after_reading_this")
                .and_then(|v| {
                    serde_json::from_value(v.clone())
                        .inspect_err(|e| {
                            tracing::warn!(field = "after_reading_this", error = %e, "Parse failed");
                        })
                        .ok()
                })
                .unwrap_or_default(),
            related_playbooks: item
                .get("related_playbooks")
                .and_then(|v| {
                    serde_json::from_value(v.clone())
                        .inspect_err(|e| {
                            tracing::warn!(field = "related_playbooks", error = %e, "Parse failed");
                        })
                        .ok()
                })
                .unwrap_or_default(),
            related_code: item
                .get("related_code")
                .and_then(|v| {
                    serde_json::from_value(v.clone())
                        .inspect_err(|e| {
                            tracing::warn!(field = "related_code", error = %e, "Parse failed");
                        })
                        .ok()
                })
                .unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn to_template_data(&self) -> Value {
        let mut data = serde_json::Map::new();

        if !self.after_reading_this.is_empty() {
            data.insert(
                "AFTER_READING_THIS".to_string(),
                serde_json::json!(self.after_reading_this),
            );
        }

        if !self.related_playbooks.is_empty() {
            data.insert(
                "RELATED_PLAYBOOKS".to_string(),
                serde_json::json!(self.related_playbooks),
            );
        }

        if !self.related_code.is_empty() {
            data.insert(
                "RELATED_CODE".to_string(),
                serde_json::json!(self.related_code),
            );
        }

        if self.has_content() {
            data.insert("HAS_LEARNING_CONTENT".to_string(), Value::Bool(true));
        }

        Value::Object(data)
    }
}
