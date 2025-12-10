use anyhow::{anyhow, Result};
use serde_json::json;
use std::time::Instant;
use systemprompt_core_logging::LogLevel;
use uuid::Uuid;

use crate::models::ai::{AiMessage, AiResponse, SamplingMetadata};
use crate::models::providers::gemini::{
    GeminiContent, GeminiFunctionCall, GeminiFunctionCallingConfig, GeminiFunctionCallingMode,
    GeminiFunctionResponse, GeminiPart, GeminiRequest, GeminiResponse, GeminiToolConfig,
};
use crate::models::tools::{CallToolResult, McpTool, ToolCall};

use super::constants::tokens;
use super::provider::GeminiProvider;
use super::tool_conversion::{build_thinking_config, convert_tools, extract_tool_response};
use super::{converters, helpers};

pub async fn generate_with_tools(
    provider: &GeminiProvider,
    messages: &[AiMessage],
    tools: Vec<McpTool>,
    metadata: &SamplingMetadata,
    model: &str,
) -> Result<(AiResponse, Vec<ToolCall>)> {
    generate_with_tools_config(
        provider,
        messages,
        tools,
        metadata,
        model,
        GeminiFunctionCallingMode::Auto,
        None,
        None,
    )
    .await
}

pub async fn generate_with_tools_forced(
    provider: &GeminiProvider,
    messages: &[AiMessage],
    tools: Vec<McpTool>,
    metadata: &SamplingMetadata,
    model: &str,
    allowed_function_names: Option<Vec<String>>,
    max_output_tokens: Option<u32>,
) -> Result<(AiResponse, Vec<ToolCall>)> {
    generate_with_tools_config(
        provider,
        messages,
        tools,
        metadata,
        model,
        GeminiFunctionCallingMode::Any,
        allowed_function_names,
        max_output_tokens,
    )
    .await
}

async fn generate_with_tools_config(
    provider: &GeminiProvider,
    messages: &[AiMessage],
    tools: Vec<McpTool>,
    metadata: &SamplingMetadata,
    model: &str,
    function_calling_mode: GeminiFunctionCallingMode,
    allowed_function_names: Option<Vec<String>>,
    max_output_tokens: Option<u32>,
) -> Result<(AiResponse, Vec<ToolCall>)> {
    let start = Instant::now();
    let request_id = Uuid::new_v4();
    let logger = provider.logger();

    let contents = converters::convert_messages(messages);
    let gemini_tools = convert_tools(provider, tools.clone())?;

    let effective_max_tokens = max_output_tokens.unwrap_or(tokens::DEFAULT_MAX_OUTPUT);

    if let Some(ref log) = logger {
        log.log(
            LogLevel::Info,
            "gemini_tools",
            "Sending tool request to Gemini",
            Some(json!({
                "request_id": request_id.to_string(),
                "model": model,
                "tool_count": tools.len(),
                "message_count": messages.len(),
                "function_calling_mode": format!("{function_calling_mode:?}"),
                "max_output_tokens": effective_max_tokens
            })),
        )
        .await
        .ok();
    }

    let thinking_config = build_thinking_config(model);
    let generation_config = helpers::build_generation_config(
        metadata,
        effective_max_tokens,
        None,
        thinking_config,
    );

    let tool_config = GeminiToolConfig {
        function_calling_config: GeminiFunctionCallingConfig {
            mode: function_calling_mode,
            allowed_function_names,
        },
    };

    let request = GeminiRequest {
        contents,
        generation_config: Some(generation_config),
        safety_settings: None,
        tools: Some(gemini_tools),
        tool_config: Some(tool_config),
    };

    let response_text = helpers::send_request(
        &provider.client,
        &provider.endpoint,
        &provider.api_key,
        &request,
        model,
        "generateContent",
    )
    .await?;

    if let Some(ref log) = logger {
        log.log(
            LogLevel::Debug,
            "gemini_tools",
            "Received response from Gemini",
            Some(json!({
                "request_id": request_id.to_string(),
                "response_length": response_text.len()
            })),
        )
        .await
        .ok();
    }

    let gemini_response: GeminiResponse = match helpers::parse_response(&response_text) {
        Ok(response) => response,
        Err(e) => {
            if let Some(ref log) = logger {
                log.log(
                    LogLevel::Error,
                    "gemini_tools",
                    "Failed to parse Gemini response",
                    Some(json!({
                        "request_id": request_id.to_string(),
                        "error": e.to_string(),
                        "response_preview": response_text.chars().take(1000).collect::<String>()
                    })),
                )
                .await
                .ok();
            }
            return Err(e);
        },
    };

    let (content, tool_calls) = extract_tool_response(provider, &gemini_response, &logger).await?;

    if let Some(ref log) = logger {
        log.log(
            LogLevel::Info,
            "gemini_tools",
            "Parsed Gemini response",
            Some(json!({
                "request_id": request_id.to_string(),
                "has_text": !content.is_empty(),
                "tool_call_count": tool_calls.len(),
                "latency_ms": start.elapsed().as_millis()
            })),
        )
        .await
        .ok();
    }

    let response = helpers::build_ai_response(
        request_id,
        &gemini_response,
        "gemini",
        model,
        start,
        content,
    );

    Ok((response, tool_calls))
}

pub async fn generate_with_tool_results(
    provider: &GeminiProvider,
    conversation_history: &[AiMessage],
    tool_calls: &[ToolCall],
    tool_results: &[CallToolResult],
    metadata: &SamplingMetadata,
    model: &str,
) -> Result<AiResponse> {
    let start = Instant::now();
    let request_id = Uuid::new_v4();

    let mut contents = converters::convert_messages(conversation_history);

    let mut assistant_parts = Vec::new();
    for tool_call in tool_calls {
        assistant_parts.push(GeminiPart::FunctionCall {
            function_call: GeminiFunctionCall {
                name: tool_call.name.clone(),
                args: tool_call.arguments.clone(),
                thought_signature: None,
            },
        });
    }

    if !assistant_parts.is_empty() {
        contents.push(GeminiContent {
            role: "model".to_string(),
            parts: assistant_parts,
        });
    }

    let mut user_parts = Vec::new();
    for (tool_call, tool_result) in tool_calls.iter().zip(tool_results.iter()) {
        user_parts.push(GeminiPart::FunctionResponse {
            function_response: GeminiFunctionResponse {
                name: tool_call.name.clone(),
                response: converters::convert_tool_result_to_json(tool_result),
            },
        });
    }

    if !user_parts.is_empty() {
        contents.push(GeminiContent {
            role: "user".to_string(),
            parts: user_parts,
        });
    }

    let generation_config =
        helpers::build_generation_config(metadata, tokens::DEFAULT_MAX_OUTPUT, None, None);

    let request = GeminiRequest {
        contents,
        generation_config: Some(generation_config),
        safety_settings: None,
        tools: None,
        tool_config: None,
    };

    let response_text = helpers::send_request(
        &provider.client,
        &provider.endpoint,
        &provider.api_key,
        &request,
        model,
        "generateContent",
    )
    .await?;

    let gemini_response: GeminiResponse = helpers::parse_response(&response_text)?;

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

    let logger = provider.logger();
    if let Some(ref log) = logger {
        log.log(
            LogLevel::Debug,
            "gemini_synthesis",
            "Tool synthesis response details",
            Some(json!({
                "request_id": request_id.to_string(),
                "model": model,
                "has_content": !content.is_empty(),
                "content_length": content.len(),
                "finish_reason": finish_reason,
                "tool_call_count": tool_calls.len(),
                "tool_result_count": tool_results.len()
            })),
        )
        .await
        .ok();
    }

    Ok(helpers::build_ai_response(
        request_id,
        &gemini_response,
        "gemini",
        model,
        start,
        content,
    ))
}
