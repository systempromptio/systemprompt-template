pub mod constructor;

pub use constructor::SystemToolsServer;

use anyhow::Result;
use rmcp::{
    model::*,
    service::RequestContext,
    ErrorData as McpError, RoleServer, ServerHandler,
};
use std::path::PathBuf;

impl ServerHandler for SystemToolsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "System Tools MCP Server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                title: Some("System Tools".into()),
                website_url: None,
            },
            instructions: Some(
                "File system tools for reading, writing, editing files, and searching with glob/grep patterns. \
                All operations are restricted to configured root directories."
                    .to_string()
            ),
        }
    }

    async fn initialize(
        &self,
        request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        // Check client capabilities for roots support
        if let Some(roots_cap) = &request.capabilities.roots {
            if roots_cap.list_changed == Some(true) {
                eprintln!("[INFO] Client supports root list changes");
            }
        }

        eprintln!("[INFO] System Tools initialized for client: {:?}",
            request.client_info.name);

        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: crate::tools::register_tools(),
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name.to_string();

        crate::tools::handle_tool_call(&tool_name, request, self).await
    }
}

impl SystemToolsServer {
    /// Helper to parse file path from tool arguments
    pub fn parse_file_path(args: &serde_json::Map<String, serde_json::Value>, key: &str) -> Result<PathBuf, McpError> {
        args.get(key)
            .and_then(|v| v.as_str())
            .map(PathBuf::from)
            .ok_or_else(|| McpError::invalid_params(format!("Missing or invalid '{key}' parameter"), None))
    }

    /// Helper to parse optional string parameter
    pub fn parse_optional_string(args: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<String> {
        args.get(key).and_then(|v| v.as_str()).map(String::from)
    }

    /// Helper to parse optional integer parameter
    pub fn parse_optional_i64(args: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<i64> {
        args.get(key).and_then(|v| v.as_i64())
    }

    /// Helper to parse optional boolean parameter
    pub fn parse_optional_bool(args: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<bool> {
        args.get(key).and_then(|v| v.as_bool())
    }
}
