use axum::http::{HeaderMap, HeaderValue};
use systemprompt_identifiers::{ContextId, SessionId, TraceId, UserId};

#[derive(Debug, Clone, Copy)]
pub struct HeaderInjector;

impl HeaderInjector {
    pub const HEADER_SESSION_ID: &'static str = "x-session-id";
    pub const HEADER_USER_ID: &'static str = "x-user-id";
    pub const HEADER_TRACE_ID: &'static str = "x-trace-id";
    pub const HEADER_CONTEXT_ID: &'static str = "x-context-id";
    pub const HEADER_AGENT_NAME: &'static str = "x-agent-name";

    pub fn inject_session_id(headers: &mut HeaderMap, session_id: &SessionId) -> Result<(), ()> {
        Self::inject_header(headers, Self::HEADER_SESSION_ID, session_id.as_str())
    }

    pub fn inject_user_id(headers: &mut HeaderMap, user_id: &UserId) -> Result<(), ()> {
        Self::inject_header(headers, Self::HEADER_USER_ID, user_id.as_str())
    }

    pub fn inject_trace_id(headers: &mut HeaderMap, trace_id: &TraceId) -> Result<(), ()> {
        Self::inject_header(headers, Self::HEADER_TRACE_ID, trace_id.as_str())
    }

    pub fn inject_context_id(headers: &mut HeaderMap, context_id: &ContextId) -> Result<(), ()> {
        if context_id.as_str().is_empty() {
            return Ok(());
        }
        Self::inject_header(headers, Self::HEADER_CONTEXT_ID, context_id.as_str())
    }

    pub fn inject_agent_name(headers: &mut HeaderMap, agent_name: &str) -> Result<(), ()> {
        Self::inject_header(headers, Self::HEADER_AGENT_NAME, agent_name)
    }

    pub fn inject_from_request_context(
        headers: &mut HeaderMap,
        session_id: &SessionId,
        user_id: &UserId,
        trace_id: &TraceId,
        context_id: &ContextId,
        agent_name: &str,
    ) -> Result<(), ()> {
        Self::inject_session_id(headers, session_id)?;
        Self::inject_user_id(headers, user_id)?;
        Self::inject_trace_id(headers, trace_id)?;
        Self::inject_context_id(headers, context_id)?;
        Self::inject_agent_name(headers, agent_name)?;
        Ok(())
    }

    fn inject_header(headers: &mut HeaderMap, name: &'static str, value: &str) -> Result<(), ()> {
        if let Ok(header_value) = HeaderValue::from_str(value) {
            headers.insert(name, header_value);
            Ok(())
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    #[test]
    fn test_inject_session_id() {
        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        let session_id = SessionId::new("sess-123".to_string());

        HeaderInjector::inject_session_id(headers, &session_id).unwrap();

        let value = headers.get("x-session-id").unwrap().to_str().unwrap();
        assert_eq!(value, "sess-123");
    }

    #[test]
    fn test_inject_user_id() {
        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        let user_id = UserId::new("user-456".to_string());

        HeaderInjector::inject_user_id(headers, &user_id).unwrap();

        let value = headers.get("x-user-id").unwrap().to_str().unwrap();
        assert_eq!(value, "user-456");
    }

    #[test]
    fn test_inject_trace_id() {
        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        let trace_id = TraceId::new("trace-789".to_string());

        HeaderInjector::inject_trace_id(headers, &trace_id).unwrap();

        let value = headers.get("x-trace-id").unwrap().to_str().unwrap();
        assert_eq!(value, "trace-789");
    }

    #[test]
    fn test_inject_context_id() {
        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        let context_id = ContextId::new("ctx-abc".to_string());

        HeaderInjector::inject_context_id(headers, &context_id).unwrap();

        let value = headers.get("x-context-id").unwrap().to_str().unwrap();
        assert_eq!(value, "ctx-abc");
    }

    #[test]
    fn test_inject_context_id_empty_skipped() {
        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        let context_id = ContextId::new("".to_string());

        HeaderInjector::inject_context_id(headers, &context_id).unwrap();

        assert!(headers.get("x-context-id").is_none());
    }

    #[test]
    fn test_inject_agent_name() {
        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();

        HeaderInjector::inject_agent_name(headers, "test_agent").unwrap();

        let value = headers.get("x-agent-name").unwrap().to_str().unwrap();
        assert_eq!(value, "test_agent");
    }

    #[test]
    fn test_inject_from_request_context() {
        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();

        let session_id = SessionId::new("sess-123".to_string());
        let user_id = UserId::new("user-456".to_string());
        let trace_id = TraceId::new("trace-789".to_string());
        let context_id = ContextId::new("ctx-abc".to_string());

        HeaderInjector::inject_from_request_context(
            headers,
            &session_id,
            &user_id,
            &trace_id,
            &context_id,
            "test_agent",
        )
        .unwrap();

        assert_eq!(
            headers.get("x-session-id").unwrap().to_str().unwrap(),
            "sess-123"
        );
        assert_eq!(
            headers.get("x-user-id").unwrap().to_str().unwrap(),
            "user-456"
        );
        assert_eq!(
            headers.get("x-trace-id").unwrap().to_str().unwrap(),
            "trace-789"
        );
        assert_eq!(
            headers.get("x-context-id").unwrap().to_str().unwrap(),
            "ctx-abc"
        );
        assert_eq!(
            headers.get("x-agent-name").unwrap().to_str().unwrap(),
            "test_agent"
        );
    }
}
