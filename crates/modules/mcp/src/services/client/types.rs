use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConnectionResult {
    pub service_name: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub connection_time_ms: u32,
    pub server_info: Option<McpProtocolInfo>,
    pub tools_count: usize,
    pub validation_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpProtocolInfo {
    pub server_name: String,
    pub version: String,
    pub protocol_version: String,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub success: bool,
    pub error_message: Option<String>,
    pub tools_count: usize,
    pub validation_type: String,
}

#[derive(Debug, Clone)]
pub struct ToolExecutionWithId {
    pub result: rmcp::model::CallToolResult,
    pub mcp_execution_id: String,
}

impl McpConnectionResult {
    pub const fn is_healthy(&self) -> bool {
        self.success && self.connection_time_ms < 2000
    }

    pub fn health_status(&self) -> &'static str {
        match self.validation_type.as_str() {
            "mcp_validated" => {
                if self.connection_time_ms < 1000 {
                    "healthy"
                } else {
                    "slow"
                }
            },
            "auth_required" | "no_tools" => "auth_required",
            "tools_request_failed" | "connection_failed" | "port_unavailable" | "timeout" => {
                "unhealthy"
            },
            _ => "unknown",
        }
    }

    pub fn status_description(&self) -> String {
        match self.validation_type.as_str() {
            "mcp_validated" => format!("MCP validated with {} tools", self.tools_count),
            "auth_required" => "Port responding, OAuth authentication required".to_string(),
            "no_tools" => "Connected but no tools returned (likely requires auth)".to_string(),
            "tools_request_failed" => {
                let error = self
                    .error_message
                    .as_deref()
                    .filter(|e| !e.is_empty())
                    .unwrap_or("[no error message]");
                format!("Tools request failed: {error}")
            },
            "connection_failed" => {
                let error = self
                    .error_message
                    .as_deref()
                    .filter(|e| !e.is_empty())
                    .unwrap_or("[no error message]");
                format!("Connection failed: {error}")
            },
            "port_unavailable" => "Port not responding".to_string(),
            "timeout" => "Connection timeout".to_string(),
            _ => "Unknown validation result".to_string(),
        }
    }
}
