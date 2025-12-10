//! Request Logging
//!
//! Logging helpers for AI request/response tracking.
//! Format: `{action} | {key}={value}, {key}={value}`

use crate::models::ai::{AiRequest, AiResponse};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::RequestContext;
use uuid::Uuid;

pub async fn log_request_start(
    logger: &LogService,
    request_id: Uuid,
    request: &AiRequest,
    provider_name: &str,
    model: &str,
    ctx: &RequestContext,
) {
    logger
        .info(
            "ai",
            &format!(
                "AI request started | request_id={}, provider={}, model={}, messages={}, \
                 user_id={}, trace_id={}",
                request_id,
                provider_name,
                model,
                request.messages.len(),
                ctx.user_id().as_str(),
                ctx.trace_id()
            ),
        )
        .await
        .ok();
}

pub async fn log_request_success(logger: &LogService, response: &AiResponse) {
    logger
        .info(
            "ai",
            &format!(
                "AI request completed | request_id={}, chars={}, tokens={}, latency_ms={}",
                response.request_id,
                response.content.len(),
                response.tokens_used.unwrap_or(0),
                response.latency_ms
            ),
        )
        .await
        .ok();
}

pub async fn log_request_error(
    logger: &LogService,
    request_id: Uuid,
    provider_name: &str,
    latency_ms: u64,
    error: &anyhow::Error,
) {
    logger
        .error(
            "ai",
            &format!(
                "AI request failed | request_id={}, provider={}, latency_ms={}, error={}",
                request_id, provider_name, latency_ms, error
            ),
        )
        .await
        .ok();
}

pub async fn log_tooled_request_start(
    logger: &LogService,
    request_id: Uuid,
    request: &AiRequest,
    provider_name: &str,
    model: &str,
    ctx: &RequestContext,
) {
    let tools = request.tools.as_deref().unwrap_or(&[]);

    logger
        .info(
            "ai",
            &format!(
                "AI tooled request started | request_id={}, provider={}, model={}, messages={}, \
                 tools={}, user_id={}, trace_id={}",
                request_id,
                provider_name,
                model,
                request.messages.len(),
                tools.len(),
                ctx.user_id().as_str(),
                ctx.trace_id()
            ),
        )
        .await
        .ok();
}

pub async fn log_ai_response(logger: &LogService, response: &AiResponse, tool_call_count: usize) {
    logger
        .info(
            "ai",
            &format!(
                "AI response received | request_id={}, chars={}, tool_calls={}",
                response.request_id,
                response.content.len(),
                tool_call_count
            ),
        )
        .await
        .ok();

    if tool_call_count == 0 && !response.content.is_empty() {
        logger
            .warn(
                "ai",
                &format!(
                    "AI text response | request_id={}, expected=tool_call, chars={}",
                    response.request_id,
                    response.content.len()
                ),
            )
            .await
            .ok();
    }
}

pub async fn log_tooled_response(logger: &LogService, response: &AiResponse) {
    logger
        .info(
            "ai",
            &format!(
                "AI tooled response | request_id={}, chars={}, tokens={}, tool_calls={}, \
                 latency_ms={}",
                response.request_id,
                response.content.len(),
                response.tokens_used.unwrap_or(0),
                response.tool_calls.len(),
                response.latency_ms
            ),
        )
        .await
        .ok();
}
