use anyhow::{anyhow, Result};
use reqwest::Client;
use std::time::Instant;
use uuid::Uuid;

use crate::models::ai::{AiResponse, SamplingMetadata};
use crate::models::providers::gemini::{
    GeminiGenerationConfig, GeminiRequest, GeminiResponse, GeminiThinkingConfig,
    GeminiUsageMetadata,
};

pub fn build_generation_config(
    metadata: &SamplingMetadata,
    max_output_tokens: u32,
    response_format: Option<(String, serde_json::Value)>,
    thinking_config: Option<GeminiThinkingConfig>,
) -> GeminiGenerationConfig {
    GeminiGenerationConfig {
        temperature: metadata.temperature,
        top_p: metadata.top_p,
        top_k: metadata.top_k,
        max_output_tokens: Some(max_output_tokens),
        stop_sequences: metadata.stop_sequences.clone(),
        response_mime_type: response_format.as_ref().map(|(mime, _)| mime.clone()),
        response_schema: response_format.map(|(_, schema)| schema),
        response_modalities: None,
        image_config: None,
        thinking_config,
    }
}

pub fn build_url(endpoint: &str, model: &str, api_key: &str, method: &str) -> String {
    format!("{}/models/{}:{}?key={}", endpoint, model, method, api_key)
}

pub async fn send_request(
    client: &Client,
    endpoint: &str,
    api_key: &str,
    request: &GeminiRequest,
    model: &str,
    method: &str,
) -> Result<String> {
    let url = build_url(endpoint, model, api_key, method);
    let response = client.post(&url).json(request).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(anyhow!("Gemini API error ({status}): {error_text}"));
    }

    Ok(response.text().await?)
}

pub fn parse_response<T: serde::de::DeserializeOwned>(response_text: &str) -> Result<T> {
    serde_json::from_str(response_text).map_err(|e| {
        anyhow!(
            "Failed to parse Gemini response: {}. Preview: {}",
            e,
            &response_text.chars().take(500).collect::<String>()
        )
    })
}

pub fn extract_token_usage(
    usage: Option<GeminiUsageMetadata>,
) -> (Option<u32>, Option<u32>, Option<u32>) {
    usage
        .map(|u| {
            (
                Some(u.total_token_count),
                Some(u.prompt_token_count),
                u.candidates_token_count,
            )
        })
        .unwrap_or((None, None, None))
}

pub fn build_ai_response(
    request_id: Uuid,
    gemini_response: &GeminiResponse,
    provider_name: &str,
    model: &str,
    start: Instant,
    content: String,
) -> AiResponse {
    let candidate = gemini_response.candidates.first();
    let (tokens_used, input_tokens, output_tokens) =
        extract_token_usage(gemini_response.usage_metadata);

    AiResponse {
        request_id,
        content,
        provider: provider_name.to_string(),
        model: model.to_string(),
        finish_reason: candidate.and_then(|c| c.finish_reason.clone()),
        tokens_used,
        input_tokens,
        output_tokens,
        cache_hit: false,
        cache_read_tokens: None,
        cache_creation_tokens: None,
        is_streaming: false,
        latency_ms: start.elapsed().as_millis() as u64,
        tool_calls: Vec::new(),
        tool_results: Vec::new(),
    }
}
