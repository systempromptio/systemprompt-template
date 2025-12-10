use crate::artifacts::metadata::ExecutionMetadata;
use crate::artifacts::traits::Artifact;
use crate::artifacts::types::ArtifactType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

fn default_artifact_type() -> String {
    "text".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TextArtifact {
    #[serde(rename = "x-artifact-type")]
    #[serde(default = "default_artifact_type")]
    pub artifact_type: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip)]
    #[schemars(skip)]
    metadata: ExecutionMetadata,
}

impl TextArtifact {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            artifact_type: "text".to_string(),
            content: content.into(),
            title: None,
            metadata: ExecutionMetadata::default(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
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

impl Artifact for TextArtifact {
    fn artifact_type(&self) -> ArtifactType {
        ArtifactType::Text
    }

    fn to_schema(&self) -> JsonValue {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Text content"
                },
                "title": {
                    "type": "string",
                    "description": "Optional title for the text"
                }
            },
            "required": ["content"],
            "x-artifact-type": "text"
        })
    }
}
