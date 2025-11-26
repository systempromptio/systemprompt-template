use axum::http::{HeaderMap, HeaderName};
use systemprompt_core_system::RequestContext;
use systemprompt_traits::InjectContextHeaders;

pub const SESSION_ID: HeaderName = HeaderName::from_static("x-session-id");
pub const TRACE_ID: HeaderName = HeaderName::from_static("x-trace-id");
pub const USER_ID: HeaderName = HeaderName::from_static("x-user-id");
pub const CONTEXT_ID: HeaderName = HeaderName::from_static("x-context-id");
pub const TASK_ID: HeaderName = HeaderName::from_static("x-task-id");

pub const MCP_SESSION_ID_STR: &str = "Mcp-Session-Id";

#[derive(Debug, Clone, Copy)]
pub struct HeaderInjector;

impl HeaderInjector {
    pub fn inject_context(headers: &mut HeaderMap, req_ctx: &RequestContext) {
        req_ctx.inject_headers(headers);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use systemprompt_identifiers::{AgentName, ContextId, SessionId, TraceId, UserId};

    #[test]
    fn inject_context_adds_all_headers_even_for_anonymous() {
        let mut headers = HeaderMap::new();
        let req_ctx = RequestContext::new(
            SessionId::new("sess-123"),
            TraceId::new("trace-456"),
            ContextId::system(),
            AgentName::new("test-agent"),
        );

        HeaderInjector::inject_context(&mut headers, &req_ctx);

        assert_eq!(headers.get(SESSION_ID).unwrap(), "sess-123");
        assert_eq!(headers.get(TRACE_ID).unwrap(), "trace-456");
        assert!(headers.get(USER_ID).is_some());
        assert!(headers.get(CONTEXT_ID).is_some());
    }

    #[test]
    fn inject_context_adds_user_id_when_present() {
        let mut headers = HeaderMap::new();
        let req_ctx = RequestContext::new(
            SessionId::new("sess-123"),
            TraceId::new("trace-456"),
            ContextId::new("ctx-abc"),
            AgentName::new("test-agent"),
        )
        .with_user_id(UserId::new("user-789"));

        HeaderInjector::inject_context(&mut headers, &req_ctx);

        assert_eq!(headers.get(SESSION_ID).unwrap(), "sess-123");
        assert_eq!(headers.get(TRACE_ID).unwrap(), "trace-456");
        assert_eq!(headers.get(USER_ID).unwrap(), "user-789");
        assert_eq!(headers.get(CONTEXT_ID).unwrap(), "ctx-abc");
    }

    #[test]
    fn inject_context_handles_empty_headers() {
        let mut headers = HeaderMap::new();
        let req_ctx = RequestContext::new(
            SessionId::new("a"),
            TraceId::new("b"),
            ContextId::system(),
            AgentName::new("test-agent"),
        );

        HeaderInjector::inject_context(&mut headers, &req_ctx);

        assert_eq!(headers.len(), 5);
    }

    #[test]
    fn inject_context_overwrites_existing_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(SESSION_ID, HeaderValue::from_static("old-session"));

        let req_ctx = RequestContext::new(
            SessionId::new("new-session"),
            TraceId::new("trace-123"),
            ContextId::system(),
            AgentName::new("test-agent"),
        );

        HeaderInjector::inject_context(&mut headers, &req_ctx);

        assert_eq!(headers.get(SESSION_ID).unwrap(), "new-session");
    }
}
