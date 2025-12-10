use async_trait::async_trait;
use axum::http::HeaderMap;
use systemprompt_identifiers::{AgentName, ContextId, SessionId, TaskId, TraceId, UserId};
use systemprompt_models::auth::UserType;
use systemprompt_models::execution::{ContextExtractionError, RequestContext};

use super::traits::ContextExtractor;

/// `HeaderContextExtractor` extracts `RequestContext` from HTTP headers
///
/// Used by:
/// - MCP servers (receiving proxied requests)
/// - Standard API routes (JWT-based auth)
///
/// Required headers:
/// - x-session-id (REQUIRED)
/// - x-trace-id (REQUIRED)
/// - x-user-id (REQUIRED)
/// - x-context-id (REQUIRED)
/// - x-task-id (OPTIONAL)
///
/// Fail-fast: Returns error if any required header is missing or invalid
#[derive(Debug, Clone, Copy)]
pub struct HeaderContextExtractor;

impl HeaderContextExtractor {
    pub const fn new() -> Self {
        Self
    }

    fn extract_required_header(
        headers: &HeaderMap,
        name: &str,
    ) -> Result<String, ContextExtractionError> {
        headers
            .get(name)
            .ok_or_else(|| ContextExtractionError::MissingHeader(name.to_string()))?
            .to_str()
            .map(ToString::to_string)
            .map_err(|e| ContextExtractionError::InvalidHeaderValue {
                header: name.to_string(),
                reason: e.to_string(),
            })
    }

    fn extract_optional_header(headers: &HeaderMap, name: &str) -> Option<String> {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string)
    }
}

impl Default for HeaderContextExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContextExtractor for HeaderContextExtractor {
    async fn extract_from_headers(
        &self,
        headers: &HeaderMap,
    ) -> Result<RequestContext, ContextExtractionError> {
        let session_id_str = Self::extract_required_header(headers, "x-session-id")?;
        let trace_id_str = Self::extract_required_header(headers, "x-trace-id")?;
        let user_id_str = Self::extract_required_header(headers, "x-user-id")?;
        let context_id_str = Self::extract_required_header(headers, "x-context-id")?;
        let agent_name_str = Self::extract_required_header(headers, "x-agent-name")?;

        let mut context = RequestContext::new(
            SessionId::new(session_id_str),
            TraceId::new(trace_id_str),
            ContextId::new(context_id_str),
            AgentName::new(agent_name_str),
        )
        .with_user_id(UserId::new(user_id_str));

        if let Some(task_id_str) = Self::extract_optional_header(headers, "x-task-id") {
            context = context.with_task_id(TaskId::new(task_id_str));
        }

        Ok(context)
    }

    async fn extract_user_only(
        &self,
        headers: &HeaderMap,
    ) -> Result<RequestContext, ContextExtractionError> {
        let session_id_str = Self::extract_required_header(headers, "x-session-id")?;
        let trace_id_str = Self::extract_required_header(headers, "x-trace-id")?;
        let user_id_str = Self::extract_required_header(headers, "x-user-id")?;
        let agent_name_str = Self::extract_required_header(headers, "x-agent-name")?;

        let context = RequestContext::new(
            SessionId::new(session_id_str),
            TraceId::new(trace_id_str),
            ContextId::new(String::new()),
            AgentName::new(agent_name_str),
        )
        .with_user_id(UserId::new(user_id_str))
        .with_user_type(UserType::Standard);

        Ok(context)
    }
}
