use crate::artifacts::metadata::ExecutionMetadata;
use crate::artifacts::traits::Artifact;
use crate::artifacts::types::ArtifactType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

fn default_artifact_type() -> String {
    "list".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub title: String,
    pub summary: String,
    pub link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ListItem {
    pub fn new(
        title: impl Into<String>,
        summary: impl Into<String>,
        link: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            title: title.into(),
            summary: summary.into(),
            link: link.into(),
            uri: None,
            slug: None,
            source_id: None,
            category: None,
            description: None,
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    pub fn with_slug(mut self, slug: impl Into<String>) -> Self {
        self.slug = Some(slug.into());
        self
    }

    pub fn with_source_id(mut self, source_id: impl Into<String>) -> Self {
        self.source_id = Some(source_id.into());
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListArtifact {
    #[serde(rename = "x-artifact-type")]
    #[serde(default = "default_artifact_type")]
    pub artifact_type: String,
    pub items: Vec<ListItem>,
    pub count: usize,
    #[serde(skip)]
    #[schemars(skip)]
    metadata: ExecutionMetadata,
}

impl ListArtifact {
    pub fn new() -> Self {
        Self {
            artifact_type: "list".to_string(),
            items: Vec::new(),
            count: 0,
            metadata: ExecutionMetadata::default(),
        }
    }

    pub fn with_items(mut self, items: Vec<ListItem>) -> Self {
        self.count = items.len();
        self.items = items;
        self
    }

    pub fn with_execution_id(mut self, id: String) -> Self {
        self.metadata.execution_id = Some(id);
        self
    }

    pub fn with_skill(
        mut self,
        skill_id: impl Into<String>,
        skill_name: impl Into<String>,
    ) -> Self {
        self.metadata = self.metadata.with_skill(skill_id.into(), skill_name.into());
        self
    }
}

impl Default for ListArtifact {
    fn default() -> Self {
        Self::new()
    }
}

impl Artifact for ListArtifact {
    fn artifact_type(&self) -> ArtifactType {
        ArtifactType::List
    }

    fn to_schema(&self) -> JsonValue {
        json!({
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "description": "Array of list items",
                    "items": {
                        "type": "object",
                        "properties": {
                            "title": {
                                "type": "string",
                                "description": "Item title"
                            },
                            "summary": {
                                "type": "string",
                                "description": "Item summary"
                            },
                            "link": {
                                "type": "string",
                                "description": "Item URL (full HTTPS URL compatible with resource_loading tool's uris parameter)"
                            },
                            "uri": {
                                "type": "string",
                                "description": "Standardized URI format (tyingshoelaces://blog/slug) for use with resource_loading tool"
                            },
                            "slug": {
                                "type": "string",
                                "description": "Content slug - can be used directly with resource_loading tool as tyingshoelaces://blog/{slug}"
                            }
                        },
                        "required": ["title", "summary", "link"]
                    }
                },
                "count": {
                    "type": "integer",
                    "description": "Total number of items"
                },
                "_execution_id": {
                    "type": "string",
                    "description": "Execution ID for tracking"
                }
            },
            "required": ["items"],
            "x-artifact-type": "list"
        })
    }
}
