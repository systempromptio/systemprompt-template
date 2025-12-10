use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use systemprompt_identifiers::McpExecutionId;

use crate::execution::context::RequestContext;

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<String>")]
    pub timestamp: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
}

impl ExecutionMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_request(ctx: &RequestContext) -> Self {
        Self {
            context_id: Some(ctx.context_id().as_str().to_string()),
            trace_id: Some(ctx.trace_id().as_str().to_string()),
            session_id: Some(ctx.session_id().as_str().to_string()),
            user_id: Some(ctx.user_id().as_str().to_string()),
            task_id: ctx.task_id().map(|t| t.as_str().to_string()),
            agent_name: Some(ctx.agent_name().as_str().to_string()),
            timestamp: Some(Utc::now()),
            tool_name: None,
            skill_id: None,
            skill_name: None,
            execution_id: None,
        }
    }

    pub fn tool(mut self, name: impl Into<String>) -> Self {
        self.tool_name = Some(name.into());
        self
    }

    pub fn with_tool(self, name: impl Into<String>) -> Self {
        self.tool(name)
    }

    pub fn skill(mut self, id: impl Into<String>, name: impl Into<String>) -> Self {
        self.skill_id = Some(id.into());
        self.skill_name = Some(name.into());
        self
    }

    pub fn with_skill(self, id: impl Into<String>, name: impl Into<String>) -> Self {
        self.skill(id, name)
    }

    pub fn execution(mut self, id: impl Into<String>) -> Self {
        self.execution_id = Some(id.into());
        self
    }

    pub fn with_execution(self, id: impl Into<String>) -> Self {
        self.execution(id)
    }

    #[allow(clippy::use_self)]
    pub fn schema() -> JsonValue {
        serde_json::to_value(schemars::schema_for!(ExecutionMetadata)).unwrap_or(JsonValue::Null)
    }

    pub fn to_meta(&self) -> Option<rmcp::model::Meta> {
        serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_object().cloned())
            .map(rmcp::model::Meta)
    }
}

#[allow(clippy::trait_duplication_in_bounds)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolResponse<T: JsonSchema> {
    #[schemars(
        description = "Unique identifier for the artifact, used to reference in subsequent tool \
                       calls"
    )]
    pub artifact_id: String,

    #[schemars(description = "MCP execution ID for tracking tool execution lifecycle")]
    pub mcp_execution_id: McpExecutionId,

    pub artifact: T,

    #[serde(rename = "_metadata")]
    pub metadata: ExecutionMetadata,
}

impl<T: Serialize + JsonSchema> ToolResponse<T> {
    /// Create a new ToolResponse with an explicit McpExecutionId.
    ///
    /// The `mcp_execution_id` MUST come from
    /// `ToolUsageRepository.start_execution()` to ensure the ID in the
    /// response matches the persisted execution record in
    /// `mcp_tool_executions`.
    pub fn new(
        artifact_id: impl Into<String>,
        mcp_execution_id: McpExecutionId,
        artifact: T,
        metadata: ExecutionMetadata,
    ) -> Self {
        Self {
            artifact_id: artifact_id.into(),
            mcp_execution_id,
            artifact,
            metadata,
        }
    }

    pub fn to_json(&self) -> JsonValue {
        serde_json::to_value(self).unwrap_or(JsonValue::Null)
    }
}

#[allow(clippy::use_self)]
impl<T: JsonSchema> ToolResponse<T> {
    pub fn schema() -> JsonValue {
        serde_json::to_value(schemars::schema_for!(ToolResponse<T>)).unwrap_or(JsonValue::Null)
    }
}
