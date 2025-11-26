use crate::models::ai::{
    AiMessage, ResponseFormat, SamplingMetadata, SamplingResponse, SearchGroundedResponse,
};
use crate::models::tools::{CallToolResult, McpTool, ToolCall};
use crate::services::schema::ProviderCapabilities;
use anyhow::Result;
use async_trait::async_trait;
use futures::stream::Stream;
use rmcp::model::RawContent;
use std::pin::Pin;

#[async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;

    fn capabilities(&self) -> ProviderCapabilities;

    fn supports_model(&self, model: &str) -> bool;

    fn supports_metadata(&self, metadata: &SamplingMetadata) -> bool;

    fn default_model(&self) -> &str;

    fn get_cost_per_1k_tokens(&self, model: &str) -> f32;

    async fn sample(
        &self,
        messages: &[AiMessage],
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<SamplingResponse>;

    async fn sample_with_tools(
        &self,
        messages: &[AiMessage],
        tools: Vec<McpTool>,
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<(SamplingResponse, Vec<ToolCall>)>;

    /// Sample with conversation history including tool calls and results.
    ///
    /// **ARCHITECTURE: MCP is the narrow waist.**
    ///
    /// All tool execution flows through MCP servers and returns `rmcp::model::CallToolResult`.
    /// This method synthesizes those MCP results into natural language responses.
    ///
    /// We do NOT convert `CallToolResult` back to provider-specific formats.
    /// Instead, we extract the data and create synthesis prompts for the AI.
    ///
    /// This method is used for the second turn of multi-turn tool execution
    /// to synthesize tool results into a human-readable response.
    async fn sample_with_tool_results(
        &self,
        conversation_history: &[AiMessage],
        tool_calls: &[ToolCall],
        tool_results: &[CallToolResult],
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<SamplingResponse> {
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
            content: format!("Based on the tool results above, please provide a helpful response to the original question:\n\n{tool_summary}"),
        });

        // Use regular sampling for the synthesis
        self.sample(&messages, metadata, model).await
    }

    /// Sample with structured output format
    async fn sample_structured(
        &self,
        messages: &[AiMessage],
        metadata: &SamplingMetadata,
        model: &str,
        _response_format: &ResponseFormat,
    ) -> Result<SamplingResponse> {
        // Default implementation falls back to regular sampling
        // Providers can override for native JSON mode support
        self.sample(messages, metadata, model).await
    }

    /// Check if provider supports native JSON mode
    fn supports_json_mode(&self) -> bool {
        false
    }

    /// Check if provider supports structured output with schemas
    fn supports_structured_output(&self) -> bool {
        false
    }

    /// Stream responses from the AI provider
    async fn sample_stream(
        &self,
        _messages: &[AiMessage],
        _metadata: &SamplingMetadata,
        _model: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        // Default implementation returns an error
        Err(anyhow::anyhow!(
            "Streaming not supported by provider {}",
            self.name()
        ))
    }

    /// Stream responses with tools from the AI provider
    async fn sample_with_tools_stream(
        &self,
        _messages: &[AiMessage],
        _tools: Vec<McpTool>,
        _metadata: &SamplingMetadata,
        _model: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        // Default implementation returns an error
        Err(anyhow::anyhow!(
            "Tool streaming not supported by provider {}",
            self.name()
        ))
    }

    /// Check if provider supports streaming
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Sample with Google Search (grounded generation) and optional URL context
    /// This method enables search-only calls for research tools
    /// If urls are provided, the URL context tool is also enabled to load those pages
    async fn sample_with_google_search(
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
