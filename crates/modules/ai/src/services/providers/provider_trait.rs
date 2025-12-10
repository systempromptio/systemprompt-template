use crate::models::ai::{
    AiMessage, AiResponse, ResponseFormat, SamplingMetadata, SearchGroundedResponse,
};
use crate::models::tools::{CallToolResult, McpTool, ToolCall};
use crate::services::schema::ProviderCapabilities;
use anyhow::Result;
use async_trait::async_trait;
use futures::stream::Stream;
use rmcp::model::RawContent;
use std::pin::Pin;

#[derive(Debug, Clone, Copy)]
pub struct ModelPricing {
    pub input_cost_per_1k: f32,
    pub output_cost_per_1k: f32,
}

impl ModelPricing {
    pub const fn new(input_cost_per_1k: f32, output_cost_per_1k: f32) -> Self {
        Self {
            input_cost_per_1k,
            output_cost_per_1k,
        }
    }
}

#[async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;

    /// Allow downcasting to concrete provider types for provider-specific
    /// methods
    fn as_any(&self) -> &dyn std::any::Any;

    fn capabilities(&self) -> ProviderCapabilities;

    fn supports_model(&self, model: &str) -> bool;

    fn supports_metadata(&self, metadata: &SamplingMetadata) -> bool;

    fn default_model(&self) -> &str;

    fn get_pricing(&self, model: &str) -> ModelPricing;

    async fn generate(
        &self,
        messages: &[AiMessage],
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<AiResponse>;

    async fn generate_with_tools(
        &self,
        messages: &[AiMessage],
        tools: Vec<McpTool>,
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<(AiResponse, Vec<ToolCall>)>;

    /// Sample with conversation history including tool calls and results.
    ///
    /// **ARCHITECTURE: MCP is the narrow waist.**
    ///
    /// All tool execution flows through MCP servers and returns
    /// `rmcp::model::CallToolResult`. This method synthesizes those MCP
    /// results into natural language responses.
    ///
    /// We do NOT convert `CallToolResult` back to provider-specific formats.
    /// Instead, we extract the data and create synthesis prompts for the AI.
    ///
    /// This method is used for the second turn of multi-turn tool execution
    /// to synthesize tool results into a human-readable response.
    async fn generate_with_tool_results(
        &self,
        conversation_history: &[AiMessage],
        tool_calls: &[ToolCall],
        tool_results: &[CallToolResult],
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<AiResponse> {
        let mut messages = conversation_history.to_vec();

        let mut tool_summary = String::new();
        for (call, result) in tool_calls.iter().zip(tool_results.iter()) {
            let content_text: String = result
                .content
                .iter()
                .filter_map(|c| match &c.raw {
                    RawContent::Text(text_content) => Some(text_content.text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");

            if result.is_error.unwrap_or(false) {
                tool_summary.push_str(&format!("Tool {} failed: {}\n", call.name, content_text));
            } else {
                tool_summary.push_str(&format!("Tool {} result: {}\n", call.name, content_text));
            }
        }

        messages.push(AiMessage {
            role: crate::models::ai::MessageRole::User,
            content: format!(
                "Based on the tool results above, please provide a helpful response to the \
                 original question:\n\n{tool_summary}"
            ),
        });

        self.generate(&messages, metadata, model).await
    }

    /// Sample with structured output format (legacy)
    async fn generate_structured(
        &self,
        messages: &[AiMessage],
        metadata: &SamplingMetadata,
        model: &str,
        _response_format: &ResponseFormat,
    ) -> Result<AiResponse> {
        self.generate(messages, metadata, model).await
    }

    /// Generate structured JSON output matching the provided schema.
    ///
    /// This is the canonical method for planning and other structured data
    /// requests. Unlike function calling, this guarantees JSON output
    /// matching the schema.
    ///
    /// Returns the raw JSON string in AiResponse.content - caller is
    /// responsible for deserializing to the target type.
    async fn generate_with_schema(
        &self,
        messages: &[AiMessage],
        response_schema: serde_json::Value,
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<AiResponse>;

    /// Check if provider supports native JSON mode
    fn supports_json_mode(&self) -> bool {
        false
    }

    /// Check if provider supports structured output with schemas
    fn supports_structured_output(&self) -> bool {
        true
    }

    /// Stream responses from the AI provider
    async fn generate_stream(
        &self,
        _messages: &[AiMessage],
        _metadata: &SamplingMetadata,
        _model: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        Err(anyhow::anyhow!(
            "Streaming not supported by provider {}",
            self.name()
        ))
    }

    async fn generate_with_tools_stream(
        &self,
        _messages: &[AiMessage],
        _tools: Vec<McpTool>,
        _metadata: &SamplingMetadata,
        _model: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        Err(anyhow::anyhow!(
            "Tool streaming not supported by provider {}",
            self.name()
        ))
    }

    /// Check if provider supports streaming
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Check if provider supports Google Search grounding
    fn supports_google_search(&self) -> bool {
        false
    }

    /// Sample with Google Search (grounded generation) and optional URL context
    /// This method enables search-only calls for research tools
    /// If urls are provided, the URL context tool is also enabled to load those
    /// pages
    async fn generate_with_google_search(
        &self,
        _messages: &[AiMessage],
        _metadata: &SamplingMetadata,
        _model: &str,
        _urls: Option<Vec<String>>,
        _response_schema: Option<serde_json::Value>,
    ) -> Result<SearchGroundedResponse> {
        Err(anyhow::anyhow!(
            "Google Search not supported by provider {}",
            self.name()
        ))
    }
}
