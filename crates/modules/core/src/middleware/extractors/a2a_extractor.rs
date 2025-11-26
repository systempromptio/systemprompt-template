use async_trait::async_trait;
use axum::body::Body;
use axum::extract::Request;
use axum::http::HeaderMap;
use systemprompt_identifiers::{AgentName, ContextId, SessionId, TraceId, UserId};
use systemprompt_models::execution::{ContextExtractionError, RequestContext};

use super::traits::ContextExtractor;
use crate::middleware::sources::{HeaderSource, PayloadSource};

#[derive(Debug, Clone, Copy)]
pub struct A2aContextExtractor;

impl A2aContextExtractor {
    pub const fn new() -> Self {
        Self
    }

    fn try_from_headers(headers: &HeaderMap) -> Result<RequestContext, ContextExtractionError> {
        let session_id = HeaderSource::extract_required(headers, "x-session-id")?;
        let trace_id = HeaderSource::extract_required(headers, "x-trace-id")?;
        let user_id = HeaderSource::extract_required(headers, "x-user-id")?;
        let context_id = HeaderSource::extract_required(headers, "x-context-id")?;
        let agent_name = HeaderSource::extract_required(headers, "x-agent-name")?;

        let mut context = RequestContext::new(
            SessionId::new(session_id),
            TraceId::new(trace_id),
            ContextId::new(context_id),
            AgentName::new(agent_name),
        )
        .with_user_id(UserId::new(user_id));

        if let Some(task_id_str) = HeaderSource::extract_optional(headers, "x-task-id") {
            context = context.with_task_id(systemprompt_identifiers::TaskId::new(task_id_str));
        }

        Ok(context)
    }

    async fn try_from_payload(
        body_bytes: &[u8],
        headers: &HeaderMap,
    ) -> Result<RequestContext, ContextExtractionError> {
        let context_id = PayloadSource::extract_context_id(body_bytes).await?;

        let session_id = HeaderSource::extract_required(headers, "x-session-id")?;
        let trace_id = HeaderSource::extract_required(headers, "x-trace-id")?;
        let user_id = HeaderSource::extract_required(headers, "x-user-id")?;
        let agent_name = HeaderSource::extract_required(headers, "x-agent-name")?;

        let mut context = RequestContext::new(
            SessionId::new(session_id),
            TraceId::new(trace_id),
            ContextId::new(context_id),
            AgentName::new(agent_name),
        )
        .with_user_id(UserId::new(user_id));

        if let Some(task_id_str) = HeaderSource::extract_optional(headers, "x-task-id") {
            context = context.with_task_id(systemprompt_identifiers::TaskId::new(task_id_str));
        }

        Ok(context)
    }
}

impl Default for A2aContextExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContextExtractor for A2aContextExtractor {
    async fn extract_from_headers(
        &self,
        headers: &HeaderMap,
    ) -> Result<RequestContext, ContextExtractionError> {
        Self::try_from_headers(headers)
    }

    async fn extract_from_request(
        &self,
        request: Request<Body>,
    ) -> Result<(RequestContext, Request<Body>), ContextExtractionError> {
        let headers = request.headers().clone();

        if let Ok(context) = Self::try_from_headers(&headers) {
            return Ok((context, request));
        }

        let (body_bytes, reconstructed_request) =
            PayloadSource::read_and_reconstruct(request).await?;

        let context = Self::try_from_payload(&body_bytes, &headers).await?;

        Ok((context, reconstructed_request))
    }
}
