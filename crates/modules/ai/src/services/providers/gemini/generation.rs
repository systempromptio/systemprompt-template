use anyhow::{anyhow, Result};
use std::time::Instant;
use uuid::Uuid;

use crate::models::ai::{AiMessage, AiResponse, SamplingMetadata};
use crate::models::providers::gemini::{GeminiPart, GeminiRequest, GeminiResponse};

use super::constants::tokens;
use super::provider::GeminiProvider;
use super::{converters, helpers};

pub async fn generate(
    provider: &GeminiProvider,
    messages: &[AiMessage],
    metadata: &SamplingMetadata,
    model: &str,
) -> Result<AiResponse> {
    let start = Instant::now();
    let request_id = Uuid::new_v4();

    let contents = converters::convert_messages(messages);
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
    let content = extract_content(&gemini_response)?;

    Ok(helpers::build_ai_response(
        request_id,
        &gemini_response,
        "gemini",
        model,
        start,
        content,
    ))
}

pub async fn generate_with_schema(
    provider: &GeminiProvider,
    messages: &[AiMessage],
    response_schema: serde_json::Value,
    metadata: &SamplingMetadata,
    model: &str,
) -> Result<AiResponse> {
    let start = Instant::now();
    let request_id = Uuid::new_v4();

    let contents = converters::convert_messages(messages);
    let generation_config = helpers::build_generation_config(
        metadata,
        tokens::EXTENDED_MAX_OUTPUT,
        Some(("application/json".to_string(), response_schema)),
        None,
    );

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
    let content = extract_content(&gemini_response)?;

    Ok(helpers::build_ai_response(
        request_id,
        &gemini_response,
        "gemini",
        model,
        start,
        content,
    ))
}

fn extract_content(gemini_response: &GeminiResponse) -> Result<String> {
    let candidate = gemini_response
        .candidates
        .first()
        .ok_or_else(|| anyhow!("No response from Gemini"))?;

    if let Some(content) = &candidate.content {
        Ok(content
            .parts
            .iter()
            .filter_map(|part| match part {
                GeminiPart::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect())
    } else {
        let reason = candidate.finish_reason.as_deref().unwrap_or("UNKNOWN");
        Err(anyhow!(
            "Gemini returned no content. Finish reason: {reason}"
        ))
    }
}
