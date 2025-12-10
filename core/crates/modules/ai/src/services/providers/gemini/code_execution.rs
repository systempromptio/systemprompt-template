use anyhow::{anyhow, Result};
use std::time::Instant;
use systemprompt_core_logging::LogLevel;
use uuid::Uuid;

use crate::models::ai::{AiMessage, SamplingMetadata};
use crate::models::providers::gemini::{CodeExecution, GeminiPart, GeminiRequest, GeminiTool};

use super::constants::tokens;
use super::provider::GeminiProvider;
use super::{converters, helpers};

#[derive(Debug, Clone)]
pub struct CodeExecutionResponse {
    pub generated_code: String,
    pub execution_output: String,
    pub success: bool,
    pub error: Option<String>,
    pub latency_ms: u64,
}

pub async fn generate_with_code_execution(
    provider: &GeminiProvider,
    messages: &[AiMessage],
    metadata: &SamplingMetadata,
    model: &str,
) -> Result<CodeExecutionResponse> {
    let start = Instant::now();
    let request_id = Uuid::new_v4();
    let logger = provider.logger();

    let contents = converters::convert_messages(messages);

    let tools = vec![GeminiTool {
        function_declarations: None,
        google_search: None,
        url_context: None,
        code_execution: Some(CodeExecution {}),
    }];

    let generation_config =
        helpers::build_generation_config(metadata, tokens::EXTENDED_MAX_OUTPUT, None, None);

    if let Some(ref log) = logger {
        log.log(
            LogLevel::Info,
            "gemini_code_execution",
            "Sending code execution request",
            Some(serde_json::json!({
                "request_id": request_id.to_string(),
                "model": model
            })),
        )
        .await
        .ok();
    }

    let request = GeminiRequest {
        contents,
        generation_config: Some(generation_config),
        safety_settings: None,
        tools: Some(tools),
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

    if let Some(ref log) = logger {
        log.log(
            LogLevel::Info,
            "gemini_code_execution",
            "Received response",
            Some(serde_json::json!({
                "request_id": request_id.to_string(),
                "response_length": response_text.len()
            })),
        )
        .await
        .ok();
    }

    let gemini_response: crate::models::providers::gemini::GeminiResponse =
        helpers::parse_response(&response_text)?;

    let candidate = gemini_response
        .candidates
        .first()
        .ok_or_else(|| anyhow!("No response from Gemini for code execution"))?;

    let mut generated_code = String::new();
    let mut execution_output = String::new();
    let mut execution_success = false;
    let mut execution_error: Option<String> = None;

    if let Some(content) = &candidate.content {
        for part in &content.parts {
            match part {
                GeminiPart::ExecutableCode { executable_code } => {
                    generated_code = executable_code.code.clone();
                },
                GeminiPart::CodeExecutionResult {
                    code_execution_result,
                } => {
                    execution_success = code_execution_result.outcome == "OUTCOME_OK";
                    if let Some(output) = &code_execution_result.output {
                        execution_output = output.clone();
                    }
                    if !execution_success {
                        execution_error = Some(format!(
                            "Code execution failed: {}",
                            code_execution_result.outcome
                        ));
                    }
                },
                GeminiPart::Text { text } => {
                    if execution_output.is_empty() && !text.is_empty() {
                        if let Some(ref log) = logger {
                            log.log(
                                LogLevel::Info,
                                "gemini_code_execution",
                                "Text response (not code result)",
                                Some(serde_json::json!({
                                    "text_preview": text.chars().take(200).collect::<String>()
                                })),
                            )
                            .await
                            .ok();
                        }
                    }
                },
                _ => {},
            }
        }
    } else {
        let reason = candidate.finish_reason.as_deref().unwrap_or("UNKNOWN");
        return Err(anyhow!(
            "Gemini returned no content for code execution. Finish reason: {reason}"
        ));
    }

    let latency_ms = start.elapsed().as_millis() as u64;

    if let Some(ref log) = logger {
        log.log(
            LogLevel::Info,
            "gemini_code_execution",
            "Code execution complete",
            Some(serde_json::json!({
                "request_id": request_id.to_string(),
                "success": execution_success,
                "code_length": generated_code.len(),
                "output_length": execution_output.len(),
                "latency_ms": latency_ms
            })),
        )
        .await
        .ok();
    }

    Ok(CodeExecutionResponse {
        generated_code,
        execution_output,
        success: execution_success,
        error: execution_error,
        latency_ms,
    })
}
