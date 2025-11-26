use crate::models::ai::{AiMessage, SamplingMetadata};
use crate::models::tools::{CallToolResult, ToolCall};
use crate::services::providers::AiProvider;
use systemprompt_core_logging::LogService;

use super::fallback_generator::{FallbackGenerator, FallbackReason};
use super::synthesis_prompt::SynthesisPromptBuilder;

#[derive(Debug)]
pub enum SynthesisResult {
    Success(String),
    NeedsFallback { reason: FallbackReason },
}

#[derive(Debug, Copy, Clone)]
pub struct ResponseSynthesizer;

impl ResponseSynthesizer {
    pub const fn new() -> Self {
        Self
    }

    pub async fn synthesize_or_fallback(
        &self,
        provider: &dyn AiProvider,
        original_messages: &[AiMessage],
        tool_calls: &[ToolCall],
        tool_results: &[CallToolResult],
        metadata: &SamplingMetadata,
        model: &str,
        logger: &LogService,
    ) -> String {
        use systemprompt_core_logging::LogLevel;

        logger
            .log(
                LogLevel::Info,
                "synthesis",
                "Starting tool result synthesis",
                Some(serde_json::json!({
                    "tool_count": tool_calls.len(),
                    "result_count": tool_results.len(),
                    "model": model,
                    "conversation_length": original_messages.len()
                })),
            )
            .await
            .ok();

        let synthesis_result = self
            .attempt_synthesis(
                provider,
                original_messages,
                tool_calls,
                tool_results,
                metadata,
                model,
            )
            .await;

        match synthesis_result {
            SynthesisResult::Success(content) => {
                logger
                    .log(
                        LogLevel::Info,
                        "synthesis",
                        "Synthesis succeeded",
                        Some(serde_json::json!({
                            "strategy": "ai_synthesis",
                            "content_length": content.len(),
                            "content_preview": content.chars().take(200).collect::<String>()
                        })),
                    )
                    .await
                    .ok();
                content
            },
            SynthesisResult::NeedsFallback { reason } => {
                let (reason_str, error_msg) = match &reason {
                    FallbackReason::EmptyContent => ("empty_content", None),
                    FallbackReason::SynthesisFailed(e) => ("synthesis_error", Some(e.as_str())),
                };

                logger
                    .log(
                        LogLevel::Warn,
                        "synthesis",
                        "Synthesis failed, using fallback generator",
                        Some(serde_json::json!({
                            "strategy": "fallback_generator",
                            "reason": reason_str,
                            "error": error_msg,
                            "tool_count": tool_calls.len()
                        })),
                    )
                    .await
                    .ok();

                match &reason {
                    FallbackReason::EmptyContent => {
                        logger
                            .warn(
                                "synthesis",
                                "AI returned empty content after tool execution - this may indicate the model needs explicit synthesis instructions"
                            )
                            .await
                            .ok();
                    },
                    FallbackReason::SynthesisFailed(error) => {
                        logger
                            .error("synthesis", &format!("Synthesis API error: {error}"))
                            .await
                            .ok();
                    },
                }

                FallbackGenerator::generate(tool_calls, tool_results, reason)
            },
        }
    }

    async fn attempt_synthesis(
        &self,
        provider: &dyn AiProvider,
        original_messages: &[AiMessage],
        tool_calls: &[ToolCall],
        tool_results: &[CallToolResult],
        metadata: &SamplingMetadata,
        model: &str,
    ) -> SynthesisResult {
        match provider
            .sample_with_tool_results(original_messages, tool_calls, tool_results, metadata, model)
            .await
        {
            Ok(response) if !response.content.is_empty() => {
                return SynthesisResult::Success(response.content);
            },
            _ => {},
        }

        let mut enhanced_messages = original_messages.to_vec();
        enhanced_messages.push(SynthesisPromptBuilder::build_guidance_message(
            tool_calls,
            tool_results,
        ));

        match provider.sample(&enhanced_messages, metadata, model).await {
            Ok(response) if !response.content.is_empty() => {
                SynthesisResult::Success(response.content)
            },
            Ok(_) => SynthesisResult::NeedsFallback {
                reason: FallbackReason::EmptyContent,
            },
            Err(e) => SynthesisResult::NeedsFallback {
                reason: FallbackReason::SynthesisFailed(e.to_string()),
            },
        }
    }
}

impl Default for ResponseSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}
