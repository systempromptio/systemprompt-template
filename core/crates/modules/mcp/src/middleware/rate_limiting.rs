use rmcp::ErrorData as McpError;
use systemprompt_core_system::RequestContext;

pub async fn check(_context: &RequestContext, _requests_per_minute: u32) -> Result<(), McpError> {
    Ok(())
}
