use chrono::Utc;
use serde::{Deserialize, Serialize};
use systemprompt_identifiers::{ContextId, TaskId};
use systemprompt_traits::validation::{
    MetadataValidation, Validate, ValidationError, ValidationResult,
};

/// Metadata for A2A Artifact entities
/// Matches frontend `ArtifactMetadata` interface in web/src/types/artifact.ts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactMetadata {
    /// Type of artifact (table, chart, code, etc.)
    pub artifact_type: String,

    /// Context ID this artifact belongs to
    pub context_id: ContextId,

    /// ISO 8601 timestamp when artifact was created
    pub created_at: String,

    /// Task ID that generated this artifact - REQUIRED (artifacts belong to
    /// tasks)
    pub task_id: TaskId,

    /// Rendering hints for UI display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendering_hints: Option<serde_json::Value>,

    /// Source of artifact (e.g., "`mcp_tool`")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// MCP tool execution ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_execution_id: Option<String>,

    /// Original MCP schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_schema: Option<serde_json::Value>,

    /// Whether this is an internal tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_internal: Option<bool>,

    /// Unique fingerprint for deduplication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,

    /// Tool name that generated this artifact
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,

    /// Execution index for tool executions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_index: Option<usize>,

    /// Skill ID that generated this artifact
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<String>,

    /// Skill name that generated this artifact
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_name: Option<String>,
}

impl ArtifactMetadata {
    /// Create new artifact metadata with required fields
    pub fn new(artifact_type: String, context_id: ContextId, task_id: TaskId) -> Self {
        Self {
            artifact_type,
            context_id,
            task_id,
            created_at: Utc::now().to_rfc3339(),
            rendering_hints: None,
            source: Some("mcp_tool".to_string()),
            mcp_execution_id: None,
            mcp_schema: None,
            is_internal: None,
            fingerprint: None,
            tool_name: None,
            execution_index: None,
            skill_id: None,
            skill_name: None,
        }
    }

    pub fn with_rendering_hints(mut self, hints: serde_json::Value) -> Self {
        self.rendering_hints = Some(hints);
        self
    }

    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_mcp_execution_id(mut self, id: String) -> Self {
        self.mcp_execution_id = Some(id);
        self
    }

    pub fn with_mcp_schema(mut self, schema: serde_json::Value) -> Self {
        self.mcp_schema = Some(schema);
        self
    }

    pub const fn with_is_internal(mut self, is_internal: bool) -> Self {
        self.is_internal = Some(is_internal);
        self
    }

    pub fn with_fingerprint(mut self, fingerprint: String) -> Self {
        self.fingerprint = Some(fingerprint);
        self
    }

    pub fn with_tool_name(mut self, tool_name: String) -> Self {
        self.tool_name = Some(tool_name);
        self
    }

    pub const fn with_execution_index(mut self, index: usize) -> Self {
        self.execution_index = Some(index);
        self
    }

    pub fn with_skill_id(mut self, skill_id: String) -> Self {
        self.skill_id = Some(skill_id);
        self
    }

    pub fn with_skill_name(mut self, skill_name: String) -> Self {
        self.skill_name = Some(skill_name);
        self
    }

    pub fn with_skill(mut self, skill_id: String, skill_name: String) -> Self {
        self.skill_id = Some(skill_id);
        self.skill_name = Some(skill_name);
        self
    }

    pub fn new_validated(
        artifact_type: String,
        context_id: ContextId,
        task_id: TaskId,
    ) -> ValidationResult<Self> {
        if artifact_type.is_empty() {
            return Err(ValidationError::new(
                "artifact_type",
                "Cannot create ArtifactMetadata: artifact_type is empty",
            )
            .with_context(format!(
                "artifact_type={artifact_type:?}, context_id={context_id:?}, task_id={task_id:?}"
            )));
        }

        let metadata = Self {
            artifact_type,
            context_id,
            task_id,
            created_at: Utc::now().to_rfc3339(),
            rendering_hints: None,
            source: Some("mcp_tool".to_string()),
            mcp_execution_id: None,
            mcp_schema: None,
            is_internal: None,
            fingerprint: None,
            tool_name: None,
            execution_index: None,
            skill_id: None,
            skill_name: None,
        };

        metadata.validate()?;
        Ok(metadata)
    }
}

impl Validate for ArtifactMetadata {
    fn validate(&self) -> ValidationResult<()> {
        self.validate_required_fields()?;
        Ok(())
    }
}

impl MetadataValidation for ArtifactMetadata {
    fn required_string_fields(&self) -> Vec<(&'static str, &str)> {
        vec![
            ("artifact_type", &self.artifact_type),
            ("context_id", self.context_id.as_str()),
            ("task_id", self.task_id.as_str()),
            ("created_at", &self.created_at),
        ]
    }
}
