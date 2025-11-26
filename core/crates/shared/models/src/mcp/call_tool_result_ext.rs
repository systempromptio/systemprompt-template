use super::McpToolResultMetadata;
use anyhow::Result;
use rmcp::model::CallToolResult;

/// Extension trait for `CallToolResult` to get strongly-typed metadata
pub trait CallToolResultExt {
    /// Extract strongly-typed MCP execution metadata from _meta field
    /// Returns error if metadata is missing or invalid
    fn get_mcp_metadata(&self) -> Result<McpToolResultMetadata>;
}

impl CallToolResultExt for CallToolResult {
    fn get_mcp_metadata(&self) -> Result<McpToolResultMetadata> {
        McpToolResultMetadata::from_call_tool_result(self)
    }
}
