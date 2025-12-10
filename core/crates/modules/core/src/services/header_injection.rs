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
