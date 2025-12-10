use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::Instant;
use uuid::Uuid;

use crate::models::ai::{AiMessage, AiResponse, MessageRole, SamplingMetadata};
use crate::models::providers::anthropic::{
    AnthropicContent, AnthropicContentBlock, AnthropicMessage, AnthropicRequest, AnthropicResponse,
    AnthropicTool,
};
use crate::models::tools::{McpTool, ToolCall};
use crate::services::providers::{AiProvider, ModelPricing};
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
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

    fn get_pricing(&self, model: &str) -> ModelPricing {
        match model {
            // Claude 3 Opus: $15/1M input, $75/1M output
            "claude-3-opus-20240229" => ModelPricing::new(0.015, 0.075),
            // Claude 3.5 Sonnet: $3/1M input, $15/1M output
            "claude-3-5-sonnet-20241022" | "claude-3-5-sonnet-20240620" => {
                ModelPricing::new(0.003, 0.015)
            },
            // Claude 3 Sonnet: $3/1M input, $15/1M output
            "claude-3-sonnet-20240229" => ModelPricing::new(0.003, 0.015),
            // Claude 3.5 Haiku: $0.80/1M input, $4/1M output
            "claude-3-5-haiku-20241022" => ModelPricing::new(0.0008, 0.004),
            // Claude 3 Haiku: $0.25/1M input, $1.25/1M output
            "claude-3-haiku-20240307" => ModelPricing::new(0.00025, 0.00125),
            _ => ModelPricing::new(0.003, 0.015),
        }
    }

    async fn generate(
        &self,
        messages: &[AiMessage],
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<AiResponse> {
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
            tool_choice: None,
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
        let cache_hit = usage.cache_read_input_tokens.is_some_and(|t| t > 0);

        Ok(AiResponse {
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
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
        })
    }

    async fn generate_with_tools(
        &self,
        messages: &[AiMessage],
        tools: Vec<McpTool>,
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<(AiResponse, Vec<ToolCall>)> {
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
            tool_choice: None,
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
        let cache_hit = usage.cache_read_input_tokens.is_some_and(|t| t > 0);

        let response = AiResponse {
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
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
        };

        Ok((response, tool_calls))
    }

    async fn generate_with_schema(
        &self,
        messages: &[AiMessage],
        response_schema: serde_json::Value,
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<AiResponse> {
        use crate::models::providers::anthropic::AnthropicToolChoice;

        let start = Instant::now();
        let request_id = Uuid::new_v4();

        let (system_prompt, anthropic_messages) = Self::convert_messages(messages);

        // Anthropic's structured output uses a forced tool call with the schema
        let structured_tool = AnthropicTool {
            name: "structured_output".to_string(),
            description: Some("Return structured JSON output matching the schema".to_string()),
            input_schema: response_schema,
        };

        let request = AnthropicRequest {
            model: model.to_string(),
            messages: anthropic_messages,
            max_tokens: 8192,
            temperature: metadata.temperature,
            top_p: metadata.top_p,
            top_k: metadata.top_k,
            stop_sequences: metadata.stop_sequences.clone(),
            system: system_prompt,
            tools: Some(vec![structured_tool]),
            tool_choice: Some(AnthropicToolChoice::Tool {
                name: "structured_output".to_string(),
            }),
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

        // Extract the tool use input as the structured content
        let content = anthropic_response
            .content
            .iter()
            .find_map(|block| match block {
                AnthropicContentBlock::ToolUse { input, .. } => {
                    Some(serde_json::to_string(input).unwrap_or_default())
                },
                _ => None,
            })
            .unwrap_or_default();

        let usage = anthropic_response.usage;
        let tokens_used = Some(usage.input_tokens + usage.output_tokens);
        let cache_hit = usage.cache_read_input_tokens.is_some_and(|t| t > 0);

        Ok(AiResponse {
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
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
        })
    }
}
