use anyhow::{anyhow, Result};
use futures::stream::StreamExt;
use futures::Stream;
use std::pin::Pin;

use crate::models::ai::{AiMessage, SamplingMetadata};
use crate::models::providers::gemini::{GeminiPart, GeminiRequest, GeminiResponse};

use super::constants::tokens;
use super::provider::GeminiProvider;
use super::{converters, helpers};

pub async fn generate_stream(
    provider: &GeminiProvider,
    messages: &[AiMessage],
    metadata: &SamplingMetadata,
    model: &str,
) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
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

    let url = helpers::build_url(
        &provider.endpoint,
        model,
        &provider.api_key,
        "streamGenerateContent",
    );

    let response = provider.client.post(&url).json(&request).send().await?;

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

                    let cleaned = text
                        .trim()
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .trim();

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
