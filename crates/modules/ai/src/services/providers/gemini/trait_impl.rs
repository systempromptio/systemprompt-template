use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use crate::models::ai::{AiMessage, AiResponse, SamplingMetadata, SearchGroundedResponse};
use crate::models::tools::{CallToolResult, McpTool, ToolCall};
use crate::services::providers::{AiProvider, ModelPricing};
use crate::services::schema::ProviderCapabilities;

use super::provider::GeminiProvider;
use super::{generation, search, streaming, tools};

#[async_trait]
impl AiProvider for GeminiProvider {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::gemini()
    }

    fn supports_model(&self, model: &str) -> bool {
        matches!(
            model,
            "gemini-2.5-flash-lite"
                | "gemini-2.5-flash"
                | "gemini-2.5-pro"
                | "gemini-3-pro-preview"
                | "gemini-2.0-flash"
                | "gemini-2.0-flash-lite"
                | "gemini-1.5-flash"
                | "gemini-1.5-flash-latest"
                | "gemini-1.5-flash-8b"
        )
    }

    fn supports_metadata(&self, _metadata: &SamplingMetadata) -> bool {
        true
    }

    fn default_model(&self) -> &'static str {
        "gemini-2.5-flash-lite"
    }

    fn get_pricing(&self, model: &str) -> ModelPricing {
        match model {
            "gemini-3-pro-preview" => ModelPricing::new(0.002, 0.012),
            "gemini-2.5-pro" | "gemini-2.5-pro-preview-05-06" => ModelPricing::new(0.00125, 0.01),
            "gemini-2.5-flash"
            | "gemini-2.5-flash-preview-04-17"
            | "gemini-2.5-flash-preview-09-2025" => ModelPricing::new(0.0003, 0.0025),
            "gemini-2.5-flash-lite" | "gemini-2.5-flash-lite-preview-09-2025" => {
                ModelPricing::new(0.0001, 0.0004)
            },
            "gemini-2.0-flash" | "gemini-2.0-flash-exp" => ModelPricing::new(0.0001, 0.0004),
            "gemini-2.0-flash-lite" => ModelPricing::new(0.000075, 0.0003),
            "gemini-1.5-pro" => ModelPricing::new(0.00125, 0.005),
            "gemini-1.5-flash" | "gemini-1.5-flash-8b" | "gemini-1.5-flash-latest" => {
                ModelPricing::new(0.000075, 0.0003)
            },
            _ => ModelPricing::new(0.0001, 0.0004),
        }
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_google_search(&self) -> bool {
        self.google_search_enabled
    }

    async fn generate(
        &self,
        messages: &[AiMessage],
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<AiResponse> {
        generation::generate(self, messages, metadata, model).await
    }

    async fn generate_with_schema(
        &self,
        messages: &[AiMessage],
        response_schema: serde_json::Value,
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<AiResponse> {
        generation::generate_with_schema(self, messages, response_schema, metadata, model).await
    }

    async fn generate_with_tools(
        &self,
        messages: &[AiMessage],
        mcp_tools: Vec<McpTool>,
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<(AiResponse, Vec<ToolCall>)> {
        tools::generate_with_tools(self, messages, mcp_tools, metadata, model).await
    }

    async fn generate_with_tool_results(
        &self,
        conversation_history: &[AiMessage],
        tool_calls: &[ToolCall],
        tool_results: &[CallToolResult],
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<AiResponse> {
        tools::generate_with_tool_results(
            self,
            conversation_history,
            tool_calls,
            tool_results,
            metadata,
            model,
        )
        .await
    }

    async fn generate_stream(
        &self,
        messages: &[AiMessage],
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        streaming::generate_stream(self, messages, metadata, model).await
    }

    async fn generate_with_google_search(
        &self,
        messages: &[AiMessage],
        metadata: &SamplingMetadata,
        model: &str,
        urls: Option<Vec<String>>,
        response_schema: Option<serde_json::Value>,
    ) -> Result<SearchGroundedResponse> {
        search::generate_with_google_search(self, messages, metadata, model, urls, response_schema)
            .await
    }
}
