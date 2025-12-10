use super::response_format::{ResponseFormat, StructuredOutputOptions};
use super::sampling::SamplingMetadata;
use super::tools::McpTool;
use crate::execution::context::RequestContext;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AiRequest {
    pub messages: Vec<AiMessage>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub metadata: SamplingMetadata,
    pub tools: Option<Vec<McpTool>>,
    pub structured_output: Option<StructuredOutputOptions>,
    pub system_prompt: Option<String>,
    pub max_tokens: Option<i32>,
}

impl AiRequest {
    pub fn new(messages: Vec<AiMessage>) -> Self {
        Self {
            messages,
            ..Default::default()
        }
    }

    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_tools(mut self, tools: Vec<McpTool>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_metadata(mut self, metadata: SamplingMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub const fn with_max_tokens(mut self, max_tokens: i32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn has_tools(&self) -> bool {
        self.tools.as_ref().is_some_and(|t| !t.is_empty())
    }

    pub fn response_format(&self) -> Option<&ResponseFormat> {
        self.structured_output
            .as_ref()
            .and_then(|so| so.response_format.as_ref())
    }

    pub fn with_context(mut self, ctx: &RequestContext) -> Self {
        self.metadata.user_id = Some(ctx.user_id().clone());
        self.metadata.session_id = Some(ctx.session_id().clone());
        self.metadata.trace_id = Some(ctx.trace_id().clone());
        if let Some(task_id) = ctx.task_id() {
            self.metadata.task_id = Some(task_id.clone());
        }
        self
    }
}
