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
