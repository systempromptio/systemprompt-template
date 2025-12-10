use chrono::Utc;
use serde::{Deserialize, Serialize};
use systemprompt_traits::validation::{
    MetadataValidation, Validate, ValidationError, ValidationResult,
};

use crate::execution::ExecutionStep;

/// Special agent name constants
pub mod agent_names {
    /// System-initiated tasks (internal operations, summaries)
    pub const SYSTEM: &str = "system";
}

/// Type discriminator for tasks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    /// Direct MCP tool execution (user or agent-initiated)
    McpExecution,
    /// Agent processing a user message
    AgentMessage,
}

/// Metadata for A2A Task entities
/// Matches frontend `TaskMetadata` interface in web/src/types/task.ts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskMetadata {
    /// Task type discriminator
    pub task_type: TaskType,

    /// Agent name that processed this task
    pub agent_name: String,

    /// ISO 8601 timestamp when task was created
    pub created_at: String,

    /// ISO 8601 timestamp when task was last updated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,

    /// ISO 8601 timestamp when task execution started
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,

    /// ISO 8601 timestamp when task execution completed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,

    /// Task execution time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_time_ms: Option<i64>,

    /// MCP tool name (for direct tool execution tracking)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,

    /// MCP server name that provided the tool (for MCP executions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_server_name: Option<String>,

    /// Total input tokens used for this task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u32>,

    /// Total output tokens generated for this task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u32>,

    /// AI model used for this task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Execution steps showing the agent's work (populated on API fetch for
    /// completed tasks)
    #[serde(rename = "executionSteps", skip_serializing_if = "Option::is_none")]
    pub execution_steps: Option<Vec<ExecutionStep>>,

    /// Extensible field for future metadata
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Map<String, serde_json::Value>>,
}

impl TaskMetadata {
    /// Create task metadata for MCP tool execution
    pub fn new_mcp_execution(
        agent_name: String,
        tool_name: String,
        mcp_server_name: String,
    ) -> Self {
        Self {
            task_type: TaskType::McpExecution,
            agent_name,
            tool_name: Some(tool_name),
            mcp_server_name: Some(mcp_server_name),
            created_at: Utc::now().to_rfc3339(),
            updated_at: None,
            started_at: None,
            completed_at: None,
            execution_time_ms: None,
            input_tokens: None,
            output_tokens: None,
            model: None,
            execution_steps: None,
            extensions: None,
        }
    }

    /// Create task metadata for agent message processing
    pub fn new_agent_message(agent_name: String) -> Self {
        Self {
            task_type: TaskType::AgentMessage,
            agent_name,
            tool_name: None,
            mcp_server_name: None,
            created_at: Utc::now().to_rfc3339(),
            updated_at: None,
            started_at: None,
            completed_at: None,
            execution_time_ms: None,
            input_tokens: None,
            output_tokens: None,
            model: None,
            execution_steps: None,
            extensions: None,
        }
    }

    /// Set token usage for this task
    pub const fn with_token_usage(mut self, input_tokens: u32, output_tokens: u32) -> Self {
        self.input_tokens = Some(input_tokens);
        self.output_tokens = Some(output_tokens);
        self
    }

    /// Set the AI model used for this task
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Mark task as updated (sets `updated_at` to current time)
    pub fn with_updated_at(mut self) -> Self {
        self.updated_at = Some(Utc::now().to_rfc3339());
        self
    }

    /// Set tool name for MCP direct tool execution tracking
    pub fn with_tool_name(mut self, tool_name: impl Into<String>) -> Self {
        self.tool_name = Some(tool_name.into());
        self
    }

    /// Set execution steps (populated when fetching completed tasks from API)
    pub fn with_execution_steps(mut self, steps: Vec<ExecutionStep>) -> Self {
        self.execution_steps = Some(steps);
        self
    }

    /// Add custom extension fields
    pub fn with_extension(mut self, key: String, value: serde_json::Value) -> Self {
        self.extensions
            .get_or_insert_with(serde_json::Map::new)
            .insert(key, value);
        self
    }

    pub fn new_validated_agent_message(agent_name: String) -> ValidationResult<Self> {
        if agent_name.is_empty() {
            return Err(ValidationError::new(
                "agent_name",
                "Cannot create TaskMetadata: agent_name is empty",
            )
            .with_context(format!("agent_name={agent_name:?}")));
        }

        let metadata = Self::new_agent_message(agent_name);
        metadata.validate()?;
        Ok(metadata)
    }

    pub fn new_validated_mcp_execution(
        agent_name: String,
        tool_name: String,
        mcp_server_name: String,
    ) -> ValidationResult<Self> {
        if agent_name.is_empty() {
            return Err(ValidationError::new(
                "agent_name",
                "Cannot create TaskMetadata: agent_name is empty",
            )
            .with_context(format!("agent_name={agent_name:?}")));
        }

        if tool_name.is_empty() {
            return Err(ValidationError::new(
                "tool_name",
                "Cannot create TaskMetadata: tool_name is empty for MCP execution",
            )
            .with_context(format!("tool_name={tool_name:?}")));
        }

        let metadata = Self::new_mcp_execution(agent_name, tool_name, mcp_server_name);
        metadata.validate()?;
        Ok(metadata)
    }
}

impl Validate for TaskMetadata {
    fn validate(&self) -> ValidationResult<()> {
        self.validate_required_fields()?;
        Ok(())
    }
}

impl MetadataValidation for TaskMetadata {
    fn required_string_fields(&self) -> Vec<(&'static str, &str)> {
        vec![
            ("agent_name", &self.agent_name),
            ("created_at", &self.created_at),
        ]
    }
}
