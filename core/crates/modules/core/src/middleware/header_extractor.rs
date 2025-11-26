use async_trait::async_trait;
use axum::http::HeaderMap;
use systemprompt_identifiers::{AgentName, ContextId, SessionId, TaskId, TraceId, UserId};
use systemprompt_models::execution::{ContextExtractionError, RequestContext, RequestContextExtractor};

/// HeaderContextExtractor extracts RequestContext from HTTP headers
///
/// Used by:
/// - Agent servers (receiving proxied requests)
/// - MCP servers (receiving proxied requests)
/// - API proxy (when validating forwarded context)
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
    pub fn new() -> Self {
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
            .map(|s| s.to_string())
            .map_err(|e| ContextExtractionError::InvalidHeaderValue {
                header: name.to_string(),
                reason: e.to_string(),
            })
    }

    fn extract_optional_header(headers: &HeaderMap, name: &str) -> Option<String> {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }
}

impl Default for HeaderContextExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RequestContextExtractor for HeaderContextExtractor {
    async fn extract(&self, headers: &HeaderMap) -> Result<RequestContext, ContextExtractionError> {
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
}

use super::context::ContextExtractor;

impl ContextExtractor for HeaderContextExtractor {
    async fn extract_standard(&self, headers: &HeaderMap) -> Result<RequestContext, ContextExtractionError> {
        self.extract(headers).await
    }

    async fn extract_mcp_a2a(&self, headers: &HeaderMap) -> Result<RequestContext, ContextExtractionError> {
        self.extract(headers).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderValue, Request};

    #[tokio::test]
    async fn extract_success_with_all_required_headers() {
        let extractor = HeaderContextExtractor::new();

        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        headers.insert("x-session-id", HeaderValue::from_static("sess-123"));
        headers.insert("x-trace-id", HeaderValue::from_static("trace-456"));
        headers.insert("x-user-id", HeaderValue::from_static("user-789"));
        headers.insert("x-context-id", HeaderValue::from_static("ctx-abc"));

        let context = extractor.extract(request.headers()).await.unwrap();

        assert_eq!(context.session_id.as_str(), "sess-123");
        assert_eq!(context.trace_id.as_str(), "trace-456");
        assert_eq!(context.user_id.as_str(), "user-789");
        assert_eq!(context.context_id.as_str(), "ctx-abc");
        assert!(context.task_id.is_none());
    }

    #[tokio::test]
    async fn extract_success_with_optional_task_id() {
        let extractor = HeaderContextExtractor::new();

        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        headers.insert("x-session-id", HeaderValue::from_static("sess-123"));
        headers.insert("x-trace-id", HeaderValue::from_static("trace-456"));
        headers.insert("x-user-id", HeaderValue::from_static("user-789"));
        headers.insert("x-context-id", HeaderValue::from_static("ctx-abc"));
        headers.insert("x-task-id", HeaderValue::from_static("task-def"));

        let context = extractor.extract(request.headers()).await.unwrap();

        assert!(context.task_id.is_some());
        assert_eq!(context.task_id.unwrap().as_str(), "task-def");
    }

    #[tokio::test]
    async fn extract_fails_when_session_id_missing() {
        let extractor = HeaderContextExtractor::new();

        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        headers.insert("x-trace-id", HeaderValue::from_static("trace-456"));
        headers.insert("x-user-id", HeaderValue::from_static("user-789"));
        headers.insert("x-context-id", HeaderValue::from_static("ctx-abc"));

        let result = extractor.extract(request.headers()).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ContextExtractionError::MissingHeader(name) => {
                assert_eq!(name, "x-session-id");
            }
            _ => panic!("Expected MissingHeader error"),
        }
    }

    #[tokio::test]
    async fn extract_fails_when_user_id_missing() {
        let extractor = HeaderContextExtractor::new();

        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        headers.insert("x-session-id", HeaderValue::from_static("sess-123"));
        headers.insert("x-trace-id", HeaderValue::from_static("trace-456"));
        headers.insert("x-context-id", HeaderValue::from_static("ctx-abc"));

        let result = extractor.extract(request.headers()).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ContextExtractionError::MissingHeader(name) => {
                assert_eq!(name, "x-user-id");
            }
            _ => panic!("Expected MissingHeader error"),
        }
    }

    #[tokio::test]
    async fn extract_fails_when_context_id_missing() {
        let extractor = HeaderContextExtractor::new();

        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        headers.insert("x-session-id", HeaderValue::from_static("sess-123"));
        headers.insert("x-trace-id", HeaderValue::from_static("trace-456"));
        headers.insert("x-user-id", HeaderValue::from_static("user-789"));

        let result = extractor.extract(request.headers()).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ContextExtractionError::MissingHeader(name) => {
                assert_eq!(name, "x-context-id");
            }
            _ => panic!("Expected MissingHeader error"),
        }
    }
}
