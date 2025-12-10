use crate::artifacts::metadata::ExecutionMetadata;
use crate::artifacts::traits::Artifact;
use crate::artifacts::types::ArtifactType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct PresentationCardResponse {
    #[serde(rename = "x-artifact-type")]
    pub artifact_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    pub sections: Vec<CardSection>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ctas: Vec<CardCta>,
    pub theme: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CardSection {
    pub heading: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

impl CardSection {
    pub fn new(heading: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            heading: heading.into(),
            content: content.into(),
            icon: None,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CardCta {
    pub id: String,
    pub label: String,
    pub message: String,
    pub variant: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

impl CardCta {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        message: impl Into<String>,
        variant: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            message: message.into(),
            variant: variant.into(),
            icon: None,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PresentationCardArtifact {
    #[serde(rename = "x-artifact-type")]
    #[serde(default = "default_card_artifact_type")]
    pub artifact_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    pub sections: Vec<CardSection>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ctas: Vec<CardCta>,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_name: Option<String>,
    #[serde(skip)]
    #[schemars(skip)]
    metadata: ExecutionMetadata,
}

fn default_theme() -> String {
    "gradient".to_string()
}

fn default_card_artifact_type() -> String {
    "presentation_card".to_string()
}

impl PresentationCardArtifact {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            artifact_type: "presentation_card".to_string(),
            title: title.into(),
            subtitle: None,
            sections: Vec::new(),
            ctas: Vec::new(),
            theme: default_theme(),
            execution_id: None,
            skill_id: None,
            skill_name: None,
            metadata: ExecutionMetadata::default(),
        }
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn with_sections(mut self, sections: Vec<CardSection>) -> Self {
        self.sections = sections;
        self
    }

    pub fn add_section(mut self, section: CardSection) -> Self {
        self.sections.push(section);
        self
    }

    pub fn with_ctas(mut self, ctas: Vec<CardCta>) -> Self {
        self.ctas = ctas;
        self
    }

    pub fn add_cta(mut self, cta: CardCta) -> Self {
        self.ctas.push(cta);
        self
    }

    pub fn with_theme(mut self, theme: impl Into<String>) -> Self {
        self.theme = theme.into();
        self
    }

    pub fn with_execution_id(mut self, id: String) -> Self {
        self.execution_id = Some(id.clone());
        self.metadata.execution_id = Some(id);
        self
    }

    pub fn with_skill(
        mut self,
        skill_id: impl Into<String>,
        skill_name: impl Into<String>,
    ) -> Self {
        self.skill_id = Some(skill_id.into());
        self.skill_name = Some(skill_name.into());
        self
    }
}

impl Artifact for PresentationCardArtifact {
    fn artifact_type(&self) -> ArtifactType {
        ArtifactType::PresentationCard
    }

    fn to_schema(&self) -> JsonValue {
        json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Card title"
                },
                "subtitle": {
                    "type": "string",
                    "description": "Card subtitle"
                },
                "sections": {
                    "type": "array",
                    "description": "Content sections",
                    "items": {
                        "type": "object",
                        "properties": {
                            "heading": {"type": "string"},
                            "content": {"type": "string"},
                            "icon": {"type": "string"}
                        },
                        "required": ["heading", "content"]
                    }
                },
                "ctas": {
                    "type": "array",
                    "description": "Call-to-action buttons",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "label": {"type": "string"},
                            "message": {"type": "string"},
                            "variant": {"type": "string"},
                            "icon": {"type": "string"}
                        },
                        "required": ["id", "label", "message", "variant"]
                    }
                },
                "theme": {
                    "type": "string",
                    "description": "Card theme",
                    "default": "gradient"
                },
                "_execution_id": {
                    "type": "string",
                    "description": "Execution ID for tracking"
                }
            },
            "required": ["title", "sections"],
            "x-artifact-type": "presentation_card",
            "x-presentation-hints": {
                "theme": self.theme
            }
        })
    }
}
