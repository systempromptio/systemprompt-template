use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::Instant;
use uuid::Uuid;

use crate::models::ai::{AiMessage, MessageRole, SamplingMetadata, SamplingResponse};
use crate::models::providers::anthropic::{
    AnthropicContent, AnthropicContentBlock, AnthropicMessage, AnthropicRequest, AnthropicResponse,
    AnthropicTool,
};
use crate::models::tools::{McpTool, ToolCall};
use crate::services::providers::AiProvider;
use crate::services::schema::ProviderCapabilities;
use systemprompt_identifiers::AiToolCallId;

#[derive(Debug)]
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    endpoint: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            endpoint: "https://api.anthropic.com/v1".to_string(),
        }
    }

    pub fn with_endpoint(api_key: String, endpoint: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            endpoint,
        }
    }

    fn convert_messages(messages: &[AiMessage]) -> (Option<String>, Vec<AnthropicMessage>) {
        let mut system_prompt = None;
        let mut anthropic_messages = Vec::new();

        for msg in messages {
            match msg.role {
                MessageRole::System => {
                    system_prompt = Some(msg.content.clone());
                },
                MessageRole::User => {
                    anthropic_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: AnthropicContent::Text(msg.content.clone()),
                    });
                },
                MessageRole::Assistant => {
                    anthropic_messages.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: AnthropicContent::Text(msg.content.clone()),
                    });
                },
            }
        }

        (system_prompt, anthropic_messages)
    }

    fn convert_tools(tools: Vec<McpTool>) -> Vec<AnthropicTool> {
        tools
            .into_iter()
            .map(|tool| AnthropicTool {
                name: tool.name,
                description: tool.description,
                input_schema: tool.input_schema.unwrap_or(json!({
                    "type": "object",
                    "properties": {}
                })),
            })
            .collect()
    }
}

#[async_trait]
impl AiProvider for AnthropicProvider {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::anthropic()
    }

    fn supports_model(&self, model: &str) -> bool {
        matches!(
            model,
            "claude-3-opus-20240229"
                | "claude-3-sonnet-20240229"
                | "claude-3-haiku-20240307"
                | "claude-3-5-sonnet-20241022"
                | "claude-3-5-haiku-20241022"
        )
    }

    fn supports_metadata(&self, _metadata: &SamplingMetadata) -> bool {
        true
    }

    fn default_model(&self) -> &'static str {
        "claude-3-sonnet-20240229"
    }

    fn get_cost_per_1k_tokens(&self, model: &str) -> f32 {
        match model {
            "claude-3-opus-20240229" => 0.015,
            "claude-3-haiku-20240307" | "claude-3-5-haiku-20241022" => 0.00025,
            _ => 0.003,
        }
    }

    async fn sample(
        &self,
        messages: &[AiMessage],
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<SamplingResponse> {
        let start = Instant::now();
        let request_id = Uuid::new_v4();

        let (system_prompt, anthropic_messages) = Self::convert_messages(messages);

        let request = AnthropicRequest {
            model: model.to_string(),
            messages: anthropic_messages,
            max_tokens: 4096,
            temperature: metadata.temperature,
            top_p: metadata.top_p,
            top_k: metadata.top_k,
            stop_sequences: metadata.stop_sequences.clone(),
            system: system_prompt,
            tools: None,
        };

        let response = self
            .client
            .post(format!("{}/messages", self.endpoint))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Anthropic API error: {error_text}"));
        }

        let anthropic_response: AnthropicResponse = response.json().await?;

        let content = anthropic_response
            .content
            .iter()
            .filter_map(|block| match block {
                AnthropicContentBlock::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<String>();

        let usage = anthropic_response.usage;
        let tokens_used = Some(usage.input_tokens + usage.output_tokens);
        let cache_hit =
            usage.cache_read_input_tokens.is_some_and(|t| t > 0);

        Ok(SamplingResponse {
            request_id,
            content,
            provider: self.name().to_string(),
            model: model.to_string(),
            finish_reason: anthropic_response.stop_reason,
            tokens_used,
            input_tokens: Some(usage.input_tokens),
            output_tokens: Some(usage.output_tokens),
            cache_hit,
            cache_read_tokens: usage.cache_read_input_tokens,
            cache_creation_tokens: usage.cache_creation_input_tokens,
            is_streaming: false,
            latency_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn sample_with_tools(
        &self,
        messages: &[AiMessage],
        tools: Vec<McpTool>,
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<(SamplingResponse, Vec<ToolCall>)> {
        let start = Instant::now();
        let request_id = Uuid::new_v4();

        let (system_prompt, anthropic_messages) = Self::convert_messages(messages);
        let anthropic_tools = Self::convert_tools(tools);

        let request = AnthropicRequest {
            model: model.to_string(),
            messages: anthropic_messages,
            max_tokens: 4096,
            temperature: metadata.temperature,
            top_p: metadata.top_p,
            top_k: metadata.top_k,
            stop_sequences: metadata.stop_sequences.clone(),
            system: system_prompt,
            tools: Some(anthropic_tools),
        };

        let response = self
            .client
            .post(format!("{}/messages", self.endpoint))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Anthropic API error: {error_text}"));
        }

        let anthropic_response: AnthropicResponse = response.json().await?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        for block in &anthropic_response.content {
            match block {
                AnthropicContentBlock::Text { text } => {
                    content.push_str(text);
                },
                AnthropicContentBlock::ToolUse { id, name, input } => {
                    tool_calls.push(ToolCall {
                        ai_tool_call_id: AiToolCallId::from(id.clone()),
                        name: name.clone(),
                        arguments: input.clone(),
                    });
                },
                AnthropicContentBlock::ToolResult { .. } => {},
            }
        }

        let usage = anthropic_response.usage;
        let tokens_used = Some(usage.input_tokens + usage.output_tokens);
        let cache_hit =
            usage.cache_read_input_tokens.is_some_and(|t| t > 0);

        let response = SamplingResponse {
            request_id,
            content,
            provider: self.name().to_string(),
            model: model.to_string(),
            finish_reason: anthropic_response.stop_reason,
            tokens_used,
            input_tokens: Some(usage.input_tokens),
            output_tokens: Some(usage.output_tokens),
            cache_hit,
            cache_read_tokens: usage.cache_read_input_tokens,
            cache_creation_tokens: usage.cache_creation_input_tokens,
            is_streaming: false,
            latency_ms: start.elapsed().as_millis() as u64,
        };

        Ok((response, tool_calls))
    }
}
