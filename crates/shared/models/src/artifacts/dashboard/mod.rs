pub mod hints;
pub mod section;
pub mod section_data;
pub mod section_types;

pub use hints::{DashboardHints, LayoutMode};
pub use section::DashboardSection;
pub use section_data::*;
pub use section_types::{LayoutWidth, SectionLayout, SectionType};

use crate::artifacts::{metadata::ExecutionMetadata, traits::Artifact, types::ArtifactType};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardArtifact {
    pub title: String,
    pub description: Option<String>,
    pub sections: Vec<DashboardSection>,
    #[serde(skip)]
    hints: DashboardHints,
    #[serde(skip)]
    metadata: ExecutionMetadata,
}

impl DashboardArtifact {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            sections: Vec::new(),
            hints: DashboardHints::default(),
            metadata: ExecutionMetadata::default(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_sections(mut self, sections: Vec<DashboardSection>) -> Self {
        self.sections = sections;
        self
    }

    pub fn add_section(mut self, section: DashboardSection) -> Self {
        self.sections.push(section);
        self
    }

    pub const fn with_hints(mut self, hints: DashboardHints) -> Self {
        self.hints = hints;
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
            "x-artifact-type": "dashboard",
            "title": self.title,
            "sections": self.sections
        });

        if let Some(ref desc) = self.description {
            response["description"] = json!(desc);
        }

        if let Some(ref id) = self.metadata.execution_id {
            response["_execution_id"] = json!(id);
        }

        response
    }
}

impl Artifact for DashboardArtifact {
    fn artifact_type(&self) -> ArtifactType {
        ArtifactType::Dashboard
    }

    fn to_schema(&self) -> JsonValue {
        json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Dashboard title"
                },
                "description": {
                    "type": "string",
                    "description": "Dashboard description"
                },
                "sections": {
                    "type": "array",
                    "description": "Dashboard sections",
                    "items": {
                        "type": "object",
                        "properties": {
                            "section_id": {"type": "string"},
                            "title": {"type": "string"},
                            "section_type": {"type": "string"},
                            "data": {"type": "object"},
                            "layout": {"type": "object"}
                        }
                    }
                },
                "_execution_id": {
                    "type": "string",
                    "description": "Execution ID for tracking"
                }
            },
            "required": ["title", "sections"],
            "x-artifact-type": "dashboard",
            "x-dashboard-hints": self.hints.generate_schema()
        })
    }
}
