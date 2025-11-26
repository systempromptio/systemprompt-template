use super::response_format::{ResponseFormat, StructuredOutputOptions};
use super::sampling::SamplingMetadata;
use super::tools::McpTool;
use serde::{Deserialize, Serialize};
use systemprompt_identifiers::{ContextId, TaskId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub role: MessageRole,
    pub content: String,
}

impl AiMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub messages: Vec<AiMessage>,
    pub metadata: Option<SamplingMetadata>,
    pub response_format: Option<ResponseFormat>,
    pub structured_output: Option<StructuredOutputOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TooledRequest {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub messages: Vec<AiMessage>,
    pub tools: Vec<McpTool>,
    pub metadata: Option<SamplingMetadata>,
    pub response_format: Option<ResponseFormat>,
    pub structured_output: Option<StructuredOutputOptions>,
    pub context_id: ContextId,
    pub task_id: TaskId,
}
