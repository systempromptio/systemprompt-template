use anyhow::Result;
use systemprompt_core_mcp::services::client::McpClient;
use systemprompt_core_system::RequestContext;
use systemprompt_models::ai::tools::McpTool;

#[derive(Debug, Clone, Copy)]
pub struct McpClientAdapter;

impl McpClientAdapter {
    /// Connect and fetch tools from MCP server
    /// Requires RequestContext with user JWT for authentication
    pub async fn fetch_tools(
        service_id: impl Into<String>,
        context: &RequestContext,
    ) -> Result<Vec<McpTool>> {
        McpClient::list_tools(service_id, context).await
    }
}
