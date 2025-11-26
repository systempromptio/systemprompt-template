use rmcp::service::RequestContext as McpContext;
use rmcp::{ErrorData as McpError, RoleServer};

pub fn extract_bearer_token(ctx: &McpContext<RoleServer>) -> Result<Option<String>, McpError> {
    let parts = ctx
        .extensions
        .get::<http::request::Parts>()
        .ok_or_else(|| {
            McpError::invalid_request("No HTTP parts in MCP context".to_string(), None)
        })?;

    let auth_header = parts
        .headers
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    if let Some(auth) = auth_header {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            return Ok(Some(token.to_string()));
        }
    }

    Ok(None)
}
