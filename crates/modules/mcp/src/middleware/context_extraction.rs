use rmcp::service::RequestContext;
use rmcp::{ErrorData as McpError, RoleServer};
use systemprompt_core_system::RequestContext as SysRequestContext;
use systemprompt_traits::ContextPropagation;

pub fn extract_request_context(
    ctx: &RequestContext<RoleServer>,
) -> Result<SysRequestContext, McpError> {
    let parts = ctx.extensions.get::<http::request::Parts>();

    if let Some(parts) = parts {
        if let Some(request_context) = parts.extensions.get::<SysRequestContext>() {
            return Ok(request_context.clone());
        }

        return SysRequestContext::from_headers(&parts.headers)
            .map_err(|e| McpError::invalid_request(e.to_string(), None));
    }

    Err(McpError::invalid_request(
        "RequestContext missing - no axum parts in MCP context",
        None,
    ))
}
