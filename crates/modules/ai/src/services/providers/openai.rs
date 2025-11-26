use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde_json::json;
use std::pin::Pin;
use std::time::Instant;
use uuid::Uuid;

use crate::models::ai::{AiMessage, ResponseFormat, SamplingMetadata, SamplingResponse};
use crate::models::providers::openai::{
    OpenAiFunction, OpenAiJsonSchema, OpenAiRequest, OpenAiResponse, OpenAiResponseFormat,
    OpenAiTool,
};
use crate::models::tools::{McpTool, ToolCall};
use crate::services::providers::AiProvider;
use crate::services::schema::ProviderCapabilities;
use systemprompt_identifiers::AiToolCallId;

#[derive(Debug)]
pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    endpoint: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            endpoint: "https://api.openai.com/v1".to_string(),
        }
    }

    pub fn with_endpoint(api_key: String, endpoint: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            endpoint,
        }
    }

    async fn create_stream_request(
        &self,
        messages: &[AiMessage],
        metadata: &SamplingMetadata,
        model: &str,
        tools: Option<Vec<OpenAiTool>>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let openai_messages: Vec<crate::models::providers::openai::OpenAiMessage> =
            messages.iter().map(Into::into).collect();

        let mut request_body = json!({
            "model": model,
            "messages": openai_messages,
            "temperature": metadata.temperature.unwrap_or(0.8),
            "max_tokens": 4096,
            "stream": true
        });

        if let Some(tools) = tools {
            request_body["tools"] = json!(tools);
        }

        let response = self
            .client
            .post(format!("{}/chat/completions", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error ({status}): {error_text}"));
        }

        let stream = response.bytes_stream().map(|chunk| -> Result<String> {
            match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    let mut content_parts = Vec::new();

                    for line in text.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data == "[DONE]" {
                                continue;
                            }

                            if let Ok(chunk_json) = serde_json::from_str::<serde_json::Value>(data)
                            {
                                if let Some(choices) = chunk_json["choices"].as_array() {
                                    if let Some(first_choice) = choices.first() {
                                        if let Some(content) =
                                            first_choice["delta"]["content"].as_str()
                                        {
                                            content_parts.push(content.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Ok(content_parts.join(""))
                },
                Err(e) => Err(anyhow!("Stream error: {e}")),
            }
        });

        Ok(Box::pin(stream))
    }

    fn convert_tools(tools: Vec<McpTool>) -> Result<Vec<OpenAiTool>> {
        tools
            .into_iter()
            .map(|tool| {
                let input_schema = tool
                    .input_schema
                    .ok_or_else(|| anyhow!("Tool '{}' missing input_schema", tool.name))?;

                Ok(OpenAiTool {
                    r#type: "function".to_string(),
                    function: OpenAiFunction {
                        name: tool.name,
                        description: tool.description,
                        parameters: input_schema,
                    },
                })
            })
            .collect()
    }

    fn convert_response_format(
        format: &ResponseFormat,
    ) -> Result<Option<OpenAiResponseFormat>> {
        match format {
            ResponseFormat::Text => Ok(None),
            ResponseFormat::JsonObject => Ok(Some(OpenAiResponseFormat::JsonObject)),
            ResponseFormat::JsonSchema {
                schema,
                name,
                strict,
            } => {
                let schema_name = name
                    .clone()
                    .ok_or_else(|| anyhow!("JSON schema response format requires a name"))?;

                Ok(Some(OpenAiResponseFormat::JsonSchema {
                    json_schema: OpenAiJsonSchema {
                        name: schema_name,
                        schema: schema.clone(),
                        strict: *strict,
                    },
                }))
            },
        }
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::openai()
    }

    fn supports_model(&self, model: &str) -> bool {
        matches!(
            model,
            "gpt-4-turbo" | "gpt-4" | "gpt-3.5-turbo" | "gpt-4o" | "gpt-4o-mini"
        )
    }

    fn supports_metadata(&self, _metadata: &SamplingMetadata) -> bool {
        true
    }

    fn default_model(&self) -> &'static str {
        "gpt-4-turbo"
    }

    fn get_cost_per_1k_tokens(&self, model: &str) -> f32 {
        match model {
            "gpt-4" | "gpt-4-turbo" => 0.03,
            "gpt-4o" => 0.005,
            "gpt-4o-mini" | "gpt-3.5-turbo" => 0.0005,
            _ => 0.01,
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

        let openai_messages: Vec<crate::models::providers::openai::OpenAiMessage> =
            messages.iter().map(Into::into).collect();

        let request = OpenAiRequest {
            model: model.to_string(),
            messages: openai_messages,
            temperature: metadata.temperature,
            top_p: metadata.top_p,
            presence_penalty: metadata.presence_penalty,
            frequency_penalty: metadata.frequency_penalty,
            max_tokens: None,
            tools: None,
            response_format: None,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.endpoint))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error: {error_text}"));
        }

        let openai_response: OpenAiResponse = response.json().await?;

        let choice = openai_response
            .choices
            .first()
            .ok_or_else(|| anyhow!("No response from OpenAI"))?;

        let content = choice
            .message
            .content
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(String::new);

        let (tokens_used, input_tokens, output_tokens, cache_hit, cache_read_tokens) =
            if let Some(usage) = openai_response.usage {
                let cache_tokens = usage
                    .prompt_tokens_details
                    .and_then(|details| details.cached_tokens);
                let cache_hit = cache_tokens.is_some_and(|t| t > 0);
                (
                    Some(usage.total_tokens),
                    Some(usage.prompt_tokens),
                    Some(usage.completion_tokens),
                    cache_hit,
                    cache_tokens,
                )
            } else {
                (None, None, None, false, None)
            };

        Ok(SamplingResponse {
            request_id,
            content,
            provider: self.name().to_string(),
            model: model.to_string(),
            finish_reason: choice.finish_reason.clone(),
            tokens_used,
            input_tokens,
            output_tokens,
            cache_hit,
            cache_read_tokens,
            cache_creation_tokens: None,
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

        let openai_messages: Vec<crate::models::providers::openai::OpenAiMessage> =
            messages.iter().map(Into::into).collect();

        let openai_tools = Self::convert_tools(tools)?;

        let request = OpenAiRequest {
            model: model.to_string(),
            messages: openai_messages,
            temperature: metadata.temperature,
            top_p: metadata.top_p,
            presence_penalty: metadata.presence_penalty,
            frequency_penalty: metadata.frequency_penalty,
            max_tokens: None,
            tools: Some(openai_tools),
            response_format: None,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.endpoint))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error: {error_text}"));
        }

        let openai_response: OpenAiResponse = response.json().await?;

        let choice = openai_response
            .choices
            .first()
            .ok_or_else(|| anyhow!("No response from OpenAI"))?;

        let content = choice
            .message
            .content
            .clone()
            .unwrap_or_else(String::new);

        let tool_calls = choice
            .message
            .tool_calls
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|tc| {
                let arguments = serde_json::from_str::<serde_json::Value>(&tc.function.arguments)
                    .unwrap_or_else(|_| json!({}));

                ToolCall {
                    ai_tool_call_id: AiToolCallId::from(tc.id),
                    name: tc.function.name,
                    arguments,
                }
            })
            .collect();

        let (tokens_used, input_tokens, output_tokens, cache_hit, cache_read_tokens) =
            if let Some(usage) = openai_response.usage {
                let cache_tokens = usage
                    .prompt_tokens_details
                    .and_then(|details| details.cached_tokens);
                let cache_hit = cache_tokens.is_some_and(|t| t > 0);
                (
                    Some(usage.total_tokens),
                    Some(usage.prompt_tokens),
                    Some(usage.completion_tokens),
                    cache_hit,
                    cache_tokens,
                )
            } else {
                (None, None, None, false, None)
            };

        let response = SamplingResponse {
            request_id,
            content,
            provider: self.name().to_string(),
            model: model.to_string(),
            finish_reason: choice.finish_reason.clone(),
            tokens_used,
            input_tokens,
            output_tokens,
            cache_hit,
            cache_read_tokens,
            cache_creation_tokens: None,
            is_streaming: false,
            latency_ms: start.elapsed().as_millis() as u64,
        };

        Ok((response, tool_calls))
    }

    async fn sample_structured(
        &self,
        messages: &[AiMessage],
        metadata: &SamplingMetadata,
        model: &str,
        response_format: &ResponseFormat,
    ) -> Result<SamplingResponse> {
        let start = Instant::now();
        let request_id = Uuid::new_v4();

        let openai_messages: Vec<crate::models::providers::openai::OpenAiMessage> =
            messages.iter().map(Into::into).collect();

        let request = OpenAiRequest {
            model: model.to_string(),
            messages: openai_messages,
            temperature: metadata.temperature,
            top_p: metadata.top_p,
            presence_penalty: metadata.presence_penalty,
            frequency_penalty: metadata.frequency_penalty,
            max_tokens: None,
            tools: None,
            response_format: Self::convert_response_format(response_format)?,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.endpoint))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error: {error_text}"));
        }

        let openai_response: OpenAiResponse = response.json().await?;

        let choice = openai_response
            .choices
            .first()
            .ok_or_else(|| anyhow!("No response from OpenAI"))?;

        let content = choice
            .message
            .content
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(String::new);

        let (tokens_used, input_tokens, output_tokens, cache_hit, cache_read_tokens) =
            if let Some(usage) = openai_response.usage {
                let cache_tokens = usage
                    .prompt_tokens_details
                    .and_then(|details| details.cached_tokens);
                let cache_hit = cache_tokens.is_some_and(|t| t > 0);
                (
                    Some(usage.total_tokens),
                    Some(usage.prompt_tokens),
                    Some(usage.completion_tokens),
                    cache_hit,
                    cache_tokens,
                )
            } else {
                (None, None, None, false, None)
            };

        Ok(SamplingResponse {
            request_id,
            content,
            provider: self.name().to_string(),
            model: model.to_string(),
            finish_reason: choice.finish_reason.clone(),
            tokens_used,
            input_tokens,
            output_tokens,
            cache_hit,
            cache_read_tokens,
            cache_creation_tokens: None,
            is_streaming: false,
            latency_ms: start.elapsed().as_millis() as u64,
        })
    }

    fn supports_json_mode(&self) -> bool {
        true
    }

    fn supports_structured_output(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    async fn sample_stream(
        &self,
        messages: &[AiMessage],
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        self.create_stream_request(messages, metadata, model, None)
            .await
    }

    async fn sample_with_tools_stream(
        &self,
        messages: &[AiMessage],
        tools: Vec<McpTool>,
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let openai_tools = Self::convert_tools(tools)?;
        self.create_stream_request(messages, metadata, model, Some(openai_tools))
            .await
    }
}
