use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::stream::StreamExt;
use futures::Stream;
use reqwest::Client;
use serde_json::json;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use uuid::Uuid;

use crate::models::ai::{
    AiMessage, MessageRole, SamplingMetadata, SamplingResponse, SearchGroundedResponse, WebSource,
};
use crate::models::providers::gemini::{
    GeminiContent, GeminiFunctionCall, GeminiFunctionDeclaration, GeminiFunctionResponse,
    GeminiGenerationConfig, GeminiPart, GeminiRequest, GeminiResponse, GeminiTool, GoogleSearch,
    UrlContext,
};
use crate::models::tools::{CallToolResult, McpTool, ToolCall};
use crate::services::providers::AiProvider;
use crate::services::schema::{
    DiscriminatedUnion, ProviderCapabilities, SchemaTransformer, ToolNameMapper,
};
use rmcp::model::RawContent;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_identifiers::AiToolCallId;

#[derive(Debug)]
pub struct GeminiProvider {
    client: Client,
    api_key: String,
    endpoint: String,
    tool_mapper: Arc<Mutex<ToolNameMapper>>,
    db_pool: Option<DbPool>,
    google_search_enabled: bool,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            api_key,
            endpoint: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            tool_mapper: Arc::new(Mutex::new(ToolNameMapper::new())),
            db_pool: None,
            google_search_enabled: false,
        }
    }

    pub fn with_endpoint(api_key: String, endpoint: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            api_key,
            endpoint,
            tool_mapper: Arc::new(Mutex::new(ToolNameMapper::new())),
            db_pool: None,
            google_search_enabled: false,
        }
    }

    pub fn with_db_pool(mut self, db_pool: DbPool) -> Self {
        self.db_pool = Some(db_pool);
        self
    }

    pub const fn with_google_search(mut self) -> Self {
        self.google_search_enabled = true;
        self
    }

    pub const fn has_google_search(&self) -> bool {
        self.google_search_enabled
    }

    fn convert_messages(messages: &[AiMessage]) -> Vec<GeminiContent> {
        let mut contents = Vec::new();
        let mut system_content = Vec::new();

        for msg in messages {
            let role = match msg.role {
                MessageRole::System => {
                    system_content.push(msg.content.clone());
                    continue;
                },
                MessageRole::User => "user",
                MessageRole::Assistant => "model",
            }
            .to_string();

            contents.push(GeminiContent {
                role,
                parts: vec![GeminiPart::Text {
                    text: msg.content.clone(),
                }],
            });
        }

        if !system_content.is_empty() {
            contents.insert(
                0,
                GeminiContent {
                    role: "user".to_string(),
                    parts: vec![GeminiPart::Text {
                        text: system_content.join("\n"),
                    }],
                },
            );
        }

        contents
    }

    fn convert_tools(&self, tools: Vec<McpTool>) -> Result<Vec<GeminiTool>> {
        use crate::models::providers::gemini::GoogleSearch;
        use crate::services::schema::TransformedTool;
        use std::collections::HashSet;

        let transformer = SchemaTransformer::new(ProviderCapabilities::gemini());
        let mut mapper = self
            .tool_mapper
            .lock()
            .map_err(|e| anyhow!("Lock poisoned: {e}"))?;

        let transformed_tools: Vec<TransformedTool> = tools
            .into_iter()
            .map(|tool| {
                let schema = tool.input_schema.as_ref();
                let discriminator_field = schema
                    .and_then(DiscriminatedUnion::detect)
                    .map(|u| u.discriminator_field);

                let result = transformer.transform(&tool)?;

                for t in &result {
                    mapper.register_transformation(t, discriminator_field.clone());
                }

                Ok(result)
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();

        let mut seen_names = HashSet::new();
        let deduplicated_tools: Vec<_> = transformed_tools
            .into_iter()
            .filter(|tool| seen_names.insert(tool.name.clone()))
            .collect();

        let mut gemini_tools = Vec::new();

        // Gemini API does NOT support both functionDeclarations and googleSearch together
        // Tested via curl: both in same object or separate objects = "Tool use with function calling is unsupported"
        // Solution: Prefer function_declarations over google_search when MCP tools exist
        if !deduplicated_tools.is_empty() {
            // When MCP tools exist, use them (disable google_search)
            gemini_tools.push(GeminiTool {
                function_declarations: Some(
                    deduplicated_tools
                        .into_iter()
                        .map(|tool| GeminiFunctionDeclaration {
                            name: tool.name,
                            description: Some(tool.description),
                            parameters: tool.input_schema,
                        })
                        .collect(),
                ),
                google_search: None,
                url_context: None,
            });
        } else if self.google_search_enabled {
            // Only use google_search when NO MCP tools are present
            gemini_tools.push(GeminiTool {
                function_declarations: None,
                google_search: Some(GoogleSearch {}),
                url_context: None,
            });
        }

        Ok(gemini_tools)
    }

    fn convert_tool_result_to_json(tool_result: &CallToolResult) -> serde_json::Value {
        if tool_result.is_error.unwrap_or(false) {
            let error_text = tool_result
                .content
                .iter()
                .filter_map(|c| match &c.raw {
                    RawContent::Text(text_content) => Some(text_content.text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            return json!({"error": error_text});
        }

        if let Some(structured) = &tool_result.structured_content {
            return structured.clone();
        }

        let content_json: Vec<serde_json::Value> = tool_result
            .content
            .iter()
            .map(|c| match &c.raw {
                RawContent::Text(text_content) => json!({"type": "text", "text": text_content.text}),
                RawContent::Image(image_content) => {
                    json!({"type": "image", "data": image_content.data, "mimeType": image_content.mime_type})
                }
                RawContent::ResourceLink(resource) => {
                    json!({"type": "resource", "uri": resource.uri, "mimeType": resource.mime_type})
                }
                _ => json!({"type": "unknown"}),
            })
            .collect();

        json!({"content": content_json})
    }
}

#[async_trait]
impl AiProvider for GeminiProvider {
    fn name(&self) -> &'static str {
        "gemini"
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

    fn get_cost_per_1k_tokens(&self, model: &str) -> f32 {
        match model {
            "gemini-2.0-flash-lite" => 0.000_075,
            "gemini-1.5-flash-8b" => 0.00005,
            "gemini-2.5-flash" => 0.0003,
            "gemini-2.5-pro" => 0.00125,
            _ => 0.0001,
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

        let contents = Self::convert_messages(messages);

        let generation_config = GeminiGenerationConfig {
            temperature: metadata.temperature,
            top_p: metadata.top_p,
            top_k: metadata.top_k,
            max_output_tokens: Some(4096),
            stop_sequences: metadata.stop_sequences.clone(),
            response_mime_type: None,
            response_schema: None,
            response_modalities: None,
            image_config: None,
        };

        let request = GeminiRequest {
            contents,
            generation_config: Some(generation_config),
            safety_settings: None,
            tools: None,
        };

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.endpoint, model, self.api_key
        );

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!("Gemini API error ({status}): {error_text}"));
        }

        let gemini_response: GeminiResponse = response.json().await?;

        let candidate = gemini_response
            .candidates
            .first()
            .ok_or_else(|| anyhow!("No response from Gemini"))?;

        let content = if let Some(content) = &candidate.content { content
        .parts
        .iter()
        .filter_map(|part| match part {
            GeminiPart::Text { text } => Some(text.clone()),
            _ => None,
        })
        .collect::<String>() } else {
            let reason = candidate.finish_reason.as_deref().unwrap_or("UNKNOWN");
            return Err(anyhow!(
                "Gemini returned no content. Finish reason: {reason}"
            ));
        };

        let tokens_used = gemini_response.usage_metadata.map(|u| u.total_token_count);

        Ok(SamplingResponse {
            request_id,
            content,
            provider: self.name().to_string(),
            model: model.to_string(),
            finish_reason: candidate.finish_reason.clone(),
            tokens_used,
            input_tokens: None,
            output_tokens: None,
            cache_hit: false,
            cache_read_tokens: None,
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

        // Initialize logger with db_pool if available
        let logger = self
            .db_pool
            .as_ref()
            .map(|pool| LogService::system(pool.clone()));

        let contents = Self::convert_messages(messages);
        let gemini_tools = self.convert_tools(tools.clone())?;

        if let Some(ref log) = logger {
            let tools_json = serde_json::to_string_pretty(&gemini_tools)
                .unwrap_or_else(|_| "[serialization failed]".to_string());
            let messages_json = serde_json::to_string(&messages)
                .unwrap_or_else(|_| "[serialization failed]".to_string());

            if let Err(e) = log
                .log(
                    systemprompt_core_logging::LogLevel::Info,
                    "gemini_provider",
                    "Sending tool request to Gemini",
                    Some(json!({
                        "request_id": request_id.to_string(),
                        "model": model,
                        "tool_count": tools.len(),
                        "tools": tools.iter().map(|t| &t.name).collect::<Vec<_>>(),
                        "tools_schema": tools_json,
                        "messages": messages_json,
                        "message_count": messages.len()
                    })),
                )
                .await
            {
                tracing::warn!("Failed to log to database: {}", e);
            }
        }

        let generation_config = GeminiGenerationConfig {
            temperature: metadata.temperature,
            top_p: metadata.top_p,
            top_k: metadata.top_k,
            max_output_tokens: Some(4096),
            stop_sequences: metadata.stop_sequences.clone(),
            response_mime_type: None,
            response_schema: None,
            response_modalities: None,
            image_config: None,
        };

        let request = GeminiRequest {
            contents,
            generation_config: Some(generation_config),
            safety_settings: None,
            tools: Some(gemini_tools),
        };

        // EMERGENCY: Log request details to stderr
        tracing::error!(
            "GEMINI TOOL REQUEST {}: model={}, tools_count={}, messages={}",
            request_id,
            model,
            request.tools.as_ref().map_or(0, Vec::len),
            request.contents.len()
        );
        if let Some(ref tools) = request.tools {
            for tool in tools {
                if let Some(ref decls) = tool.function_declarations {
                    for decl in decls {
                        tracing::error!("  Tool: {} - {:?}", decl.name, decl.description);
                    }
                }
            }
        }

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.endpoint, model, self.api_key
        );

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!("Gemini API error ({status}): {error_text}"));
        }

        let response_text = response.text().await?;

        // EMERGENCY: Log to stderr for debugging
        tracing::error!(
            "GEMINI RAW RESPONSE for {}: {}",
            request_id,
            &response_text.chars().take(2000).collect::<String>()
        );

        // Log raw response
        if let Some(ref log) = logger {
            if let Err(e) = log
                .log(
                    systemprompt_core_logging::LogLevel::Info,
                    "gemini_provider",
                    "Received response from Gemini",
                    Some(json!({
                        "request_id": request_id.to_string(),
                        "response_length": response_text.len(),
                        "response_preview": response_text.chars().take(500).collect::<String>(),
                        "raw_response": response_text
                    })),
                )
                .await
            {
                tracing::warn!("Failed to log to database: {}", e);
            }
        }

        let gemini_response: GeminiResponse = match serde_json::from_str(&response_text) {
            Ok(response) => response,
            Err(e) => {
                if let Some(ref log) = logger {
                    if let Err(log_err) = log.log(
                        systemprompt_core_logging::LogLevel::Error,
                        "gemini_provider",
                        "Failed to parse Gemini response",
                        Some(json!({
                            "request_id": request_id.to_string(),
                            "error": e.to_string(),
                            "response_preview": response_text.chars().take(1000).collect::<String>()
                        }))
                    ).await {
                            tracing::warn!("Failed to log parse error to database: {}", log_err);
                        }
                } else {
                    tracing::error!("Failed to parse Gemini response for {}: {}", request_id, e);
                }
                return Err(anyhow!(
                    "Failed to parse Gemini response: {}. Response: {}",
                    e,
                    &response_text.chars().take(500).collect::<String>()
                ));
            },
        };

        let candidate = gemini_response
            .candidates
            .first()
            .ok_or_else(|| anyhow!("No response from Gemini"))?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        if let Some(candidate_content) = &candidate.content {
            let mapper = self
                .tool_mapper
                .lock()
                .map_err(|e| anyhow!("Lock poisoned: {e}"))?;

            for part in &candidate_content.parts {
                match part {
                    GeminiPart::Text { text } => {
                        content.push_str(text);
                    },
                    GeminiPart::FunctionCall { function_call } => {
                        let (original_name, resolved_args) = mapper
                            .resolve_tool_call(&function_call.name, function_call.args.clone());

                        tool_calls.push(ToolCall {
                            ai_tool_call_id: AiToolCallId::from(Uuid::new_v4().to_string()),
                            name: original_name,
                            arguments: resolved_args,
                        });
                    },
                    _ => {},
                }
            }
        } else {
            let reason = candidate.finish_reason.as_deref().unwrap_or("UNKNOWN");
            return Err(anyhow!(
                "Gemini returned no content. Finish reason: {reason}"
            ));
        }

        // Log parsed response
        if let Some(ref log) = logger {
            if let Err(e) = log
                .log(
                    systemprompt_core_logging::LogLevel::Info,
                    "gemini_provider",
                    "Parsed Gemini response",
                    Some(json!({
                        "request_id": request_id.to_string(),
                        "has_text": !content.is_empty(),
                        "text_length": content.len(),
                        "text_preview": content.chars().take(200).collect::<String>(),
                        "tool_call_count": tool_calls.len(),
                        "tool_calls": tool_calls,
                        "finish_reason": candidate.finish_reason,
                        "latency_ms": start.elapsed().as_millis()
                    })),
                )
                .await
            {
                tracing::warn!("Failed to log parsed response to database: {}", e);
            }
        }

        let tokens_used = gemini_response.usage_metadata.map(|u| u.total_token_count);

        let response = SamplingResponse {
            request_id,
            content,
            provider: self.name().to_string(),
            model: model.to_string(),
            finish_reason: candidate.finish_reason.clone(),
            tokens_used,
            input_tokens: None,
            output_tokens: None,
            cache_hit: false,
            cache_read_tokens: None,
            cache_creation_tokens: None,
            is_streaming: false,
            latency_ms: start.elapsed().as_millis() as u64,
        };

        Ok((response, tool_calls))
    }

    async fn sample_with_tool_results(
        &self,
        conversation_history: &[AiMessage],
        tool_calls: &[ToolCall],
        tool_results: &[CallToolResult],
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<SamplingResponse> {
        let start = Instant::now();
        let request_id = Uuid::new_v4();

        // Convert conversation history to Gemini format
        let mut contents = Self::convert_messages(conversation_history);

        // Add assistant message with tool calls
        let mut assistant_parts = Vec::new();
        for tool_call in tool_calls {
            assistant_parts.push(GeminiPart::FunctionCall {
                function_call: GeminiFunctionCall {
                    name: tool_call.name.clone(),
                    args: tool_call.arguments.clone(),
                },
            });
        }

        if !assistant_parts.is_empty() {
            contents.push(GeminiContent {
                role: "model".to_string(),
                parts: assistant_parts,
            });
        }

        // Add user message with tool responses
        let mut user_parts = Vec::new();
        for (tool_call, tool_result) in tool_calls.iter().zip(tool_results.iter()) {
            user_parts.push(GeminiPart::FunctionResponse {
                function_response: GeminiFunctionResponse {
                    name: tool_call.name.clone(),
                    response: Self::convert_tool_result_to_json(tool_result),
                },
            });
        }

        if !user_parts.is_empty() {
            contents.push(GeminiContent {
                role: "user".to_string(),
                parts: user_parts,
            });
        }

        let generation_config = GeminiGenerationConfig {
            temperature: metadata.temperature,
            top_p: metadata.top_p,
            top_k: metadata.top_k,
            max_output_tokens: Some(4096),
            stop_sequences: metadata.stop_sequences.clone(),
            response_mime_type: None,
            response_schema: None,
            response_modalities: None,
            image_config: None,
        };

        let request = GeminiRequest {
            contents,
            generation_config: Some(generation_config),
            safety_settings: None,
            tools: None, // No tools needed for synthesis
        };

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.endpoint, model, self.api_key
        );

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!(
                "Gemini API error during tool synthesis: {error_text}"
            ));
        }

        let response_text = response.text().await?;
        let gemini_response: GeminiResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                anyhow!(
                    "Failed to parse Gemini synthesis response: {}. Response: {}",
                    e,
                    &response_text.chars().take(500).collect::<String>()
                )
            })?;

        let candidate = gemini_response
            .candidates
            .first()
            .ok_or_else(|| anyhow!("No response from Gemini for tool synthesis"))?;

        let finish_reason = candidate.finish_reason.as_deref().unwrap_or("UNKNOWN");
        let mut content = String::new();

        if let Some(candidate_content) = &candidate.content {
            for part in &candidate_content.parts {
                if let GeminiPart::Text { text } = part {
                    content.push_str(text);
                }
            }
        } else {
            return Err(anyhow!(
                "Gemini returned no content after tool execution. Finish reason: {finish_reason}"
            ));
        }

        let logger = self
            .db_pool
            .as_ref()
            .map(|pool| LogService::system(pool.clone()));
        if let Some(ref log) = logger {
            use systemprompt_core_logging::LogLevel;

            if let Err(e) = log
                .log(
                    LogLevel::Debug,
                    "gemini_synthesis",
                    "Tool synthesis response details",
                    Some(json!({
                        "request_id": request_id.to_string(),
                        "model": model,
                        "has_content": !content.is_empty(),
                        "content_length": content.len(),
                        "content_preview": content.chars().take(300).collect::<String>(),
                        "finish_reason": finish_reason,
                        "tool_call_count": tool_calls.len(),
                        "tool_result_count": tool_results.len(),
                        "conversation_history_messages": conversation_history.len(),
                        "raw_response_length": response_text.len(),
                        "raw_response_preview": response_text.chars().take(500).collect::<String>()
                    })),
                )
                .await
            {
                tracing::warn!("Failed to log synthesis details to database: {}", e);
            }

            if content.is_empty() {
                if let Err(e) = log
                    .warn(
                        "gemini_synthesis",
                        &format!(
                            "Gemini returned empty synthesis response (finish_reason: {finish_reason})"
                        ),
                    )
                    .await
                {
                    tracing::warn!("Failed to log empty synthesis warning to database: {}", e);
                }
            }
        }

        let tokens_used = gemini_response.usage_metadata.map(|u| u.total_token_count);

        Ok(SamplingResponse {
            request_id,
            content,
            provider: self.name().to_string(),
            model: model.to_string(),
            finish_reason: candidate.finish_reason.clone(),
            tokens_used,
            input_tokens: None,
            output_tokens: None,
            cache_hit: false,
            cache_read_tokens: None,
            cache_creation_tokens: None,
            is_streaming: false,
            latency_ms: start.elapsed().as_millis() as u64,
        })
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
        let contents = Self::convert_messages(messages);

        let generation_config = GeminiGenerationConfig {
            temperature: metadata.temperature,
            top_p: metadata.top_p,
            top_k: metadata.top_k,
            max_output_tokens: Some(4096),
            stop_sequences: metadata.stop_sequences.clone(),
            response_mime_type: None,
            response_schema: None,
            response_modalities: None,
            image_config: None,
        };

        let request = GeminiRequest {
            contents,
            generation_config: Some(generation_config),
            safety_settings: None,
            tools: None,
        };

        let url = format!(
            "{}/models/{}:streamGenerateContent?key={}",
            self.endpoint, model, self.api_key
        );

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Gemini streaming API error: {error_text}"));
        }

        let byte_stream = response.bytes_stream();

        let text_stream = byte_stream
            .map(|result| {
                result
                    .map_err(|e| anyhow!("Stream error: {e}"))
                    .map(|bytes| {
                        let text = String::from_utf8_lossy(&bytes);

                        // Remove array brackets and split by commas
                        let cleaned = text
                            .trim()
                            .trim_start_matches('[')
                            .trim_end_matches(']')
                            .trim();

                        // Try to parse as complete JSON array first
                        if let Ok(responses) =
                            serde_json::from_str::<Vec<GeminiResponse>>(&format!("[{cleaned}]"))
                        {
                            for response in responses {
                                if let Some(candidate) = response.candidates.first() {
                                    if let Some(candidate_content) = &candidate.content {
                                        let content: String = candidate_content
                                            .parts
                                            .iter()
                                            .filter_map(|part| match part {
                                                GeminiPart::Text { text } => Some(text.clone()),
                                                _ => None,
                                            })
                                            .collect();

                                        if !content.is_empty() {
                                            return content;
                                        }
                                    }
                                }
                            }
                        }

                        // Try parsing individual JSON objects separated by commas
                        for chunk in cleaned.split("\n,\n") {
                            let trimmed = chunk.trim().trim_start_matches(',').trim();
                            if trimmed.is_empty() || !trimmed.starts_with('{') {
                                continue;
                            }

                            if let Ok(response) = serde_json::from_str::<GeminiResponse>(trimmed) {
                                if let Some(candidate) = response.candidates.first() {
                                    if let Some(candidate_content) = &candidate.content {
                                        let content: String = candidate_content
                                            .parts
                                            .iter()
                                            .filter_map(|part| match part {
                                                GeminiPart::Text { text } => Some(text.clone()),
                                                _ => None,
                                            })
                                            .collect();

                                        if !content.is_empty() {
                                            return content;
                                        }
                                    }
                                }
                            }
                        }

                        String::new()
                    })
            })
            .filter(|result| {
                futures::future::ready(result.as_ref().map(|s| !s.is_empty()).unwrap_or(true))
            });

        Ok(Box::pin(text_stream))
    }

    async fn sample_with_google_search(
        &self,
        messages: &[AiMessage],
        metadata: &SamplingMetadata,
        model: &str,
        urls: Option<Vec<String>>,
        _response_schema: Option<serde_json::Value>,
    ) -> Result<SearchGroundedResponse> {
        let start = Instant::now();
        let request_id = Uuid::new_v4();

        let contents = Self::convert_messages(messages);

        let generation_config = GeminiGenerationConfig {
            temperature: metadata.temperature,
            top_p: metadata.top_p,
            top_k: metadata.top_k,
            max_output_tokens: Some(4096),
            stop_sequences: metadata.stop_sequences.clone(),
            // IMPORTANT: Cannot use response_mime_type/response_schema with tools (googleSearch)
            // Gemini API returns 400: "Tool use with a response mime type is unsupported"
            response_mime_type: None,
            response_schema: None,
            response_modalities: None,
            image_config: None,
        };

        // Build tools array based on what's needed
        let mut gemini_tools = Vec::new();

        // Always include Google Search
        gemini_tools.push(GeminiTool {
            function_declarations: None,
            google_search: Some(GoogleSearch {}),
            url_context: None,
        });

        // If URLs provided, add URL Context tool
        if urls.is_some() {
            gemini_tools.push(GeminiTool {
                function_declarations: None,
                google_search: None,
                url_context: Some(UrlContext {}),
            });
        }

        let request = GeminiRequest {
            contents,
            generation_config: Some(generation_config),
            safety_settings: None,
            tools: Some(gemini_tools),
        };

        // EMERGENCY: Log request details to stderr
        tracing::error!(
            "GEMINI TOOL REQUEST {}: model={}, tools_count={}, messages={}",
            request_id,
            model,
            request.tools.as_ref().map_or(0, Vec::len),
            request.contents.len()
        );
        if let Some(ref tools) = request.tools {
            for tool in tools {
                if let Some(ref decls) = tool.function_declarations {
                    for decl in decls {
                        tracing::error!("  Tool: {} - {:?}", decl.name, decl.description);
                    }
                }
            }
        }

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.endpoint, model, self.api_key
        );

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!("Gemini API error ({status}): {error_text}"));
        }

        let gemini_response: GeminiResponse = response.json().await?;

        let candidate = gemini_response
            .candidates
            .first()
            .ok_or_else(|| anyhow!("No response from Gemini"))?;

        let content_text = candidate
            .content
            .as_ref()
            .and_then(|c| {
                c.parts.iter().find_map(|p| match p {
                    GeminiPart::Text { text } => Some(text.clone()),
                    _ => None,
                })
            })
            .unwrap_or_default();

        let mut sources = Vec::new();
        let mut confidence_scores = Vec::new();
        let mut web_search_queries = Vec::new();

        if let Some(grounding) = &candidate.grounding_metadata {
            for chunk in &grounding.grounding_chunks {
                sources.push(WebSource {
                    title: chunk.web.title.clone(),
                    uri: chunk.web.uri.clone(),
                    relevance: 0.85,
                });
            }

            for support in &grounding.grounding_supports {
                for score in &support.confidence_scores {
                    confidence_scores.push(*score);
                }
            }

            web_search_queries.clone_from(&grounding.web_search_queries);
        }

        // Parse URL context metadata if present
        let url_context_metadata = candidate.url_context_metadata.as_ref().map(|meta| {
            use systemprompt_models::ai::UrlMetadata;
            meta.url_metadata
                .iter()
                .map(|url_meta| UrlMetadata {
                    retrieved_url: url_meta.retrieved_url.clone(),
                    url_retrieval_status: url_meta.url_retrieval_status.clone(),
                })
                .collect()
        });

        let latency_ms = start.elapsed().as_millis() as u64;

        let finish_reason = candidate.finish_reason.clone();
        let safety_ratings = candidate.safety_ratings.as_ref().map(|ratings| {
            ratings
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "category": r.category,
                        "probability": r.probability
                    })
                })
                .collect()
        });

        Ok(SearchGroundedResponse {
            content: content_text,
            sources,
            confidence_scores,
            web_search_queries,
            url_context_metadata,
            tokens_used: gemini_response
                .usage_metadata
                .as_ref()
                .map(|u| u.total_token_count),
            latency_ms,
            finish_reason,
            safety_ratings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_search_enablement() {
        let provider = GeminiProvider::new("test_key".to_string());
        assert!(!provider.has_google_search());

        let provider_with_search = provider.with_google_search();
        assert!(provider_with_search.has_google_search());
    }

    #[test]
    fn test_tool_conversion_with_search_enabled() {
        let provider = GeminiProvider::new("test_key".to_string()).with_google_search();

        let mcp_tools = vec![];
        let gemini_tools = provider.convert_tools(mcp_tools).unwrap();

        assert_eq!(gemini_tools.len(), 1);
        assert!(gemini_tools[0].google_search.is_some());
        assert!(gemini_tools[0].function_declarations.is_none());
    }

    #[test]
    fn test_tool_conversion_with_search_disabled() {
        let provider = GeminiProvider::new("test_key".to_string());

        let mcp_tools = vec![];
        let gemini_tools = provider.convert_tools(mcp_tools).unwrap();

        assert!(gemini_tools.is_empty());
    }

    #[test]
    fn test_provider_google_search_builder_chain() {
        let provider = GeminiProvider::new("test_key".to_string()).with_google_search();

        assert!(provider.has_google_search());
        assert_eq!(provider.name(), "gemini");
    }
}
