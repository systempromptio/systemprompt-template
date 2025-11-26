use crate::artifacts::{metadata::ExecutionMetadata, traits::Artifact, types::ArtifactType};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextArtifact {
    pub content: String,
    pub title: Option<String>,
    #[serde(skip)]
    metadata: ExecutionMetadata,
}

impl TextArtifact {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
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

    pub fn to_response(&self) -> JsonValue {
        let mut response = json!({
            "x-artifact-type": "text",
            "content": self.content
        });

        if let Some(ref title) = self.title {
            response["title"] = json!(title);
        }

        if let Some(ref id) = self.metadata.execution_id {
            response["_execution_id"] = json!(id);
        }

        response
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
