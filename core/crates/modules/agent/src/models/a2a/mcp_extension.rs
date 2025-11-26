//! MCP Extension types for A2A Agent Cards
//!
//! These types define the structure of MCP server metadata
//! included in AgentCard extensions per the A2A protocol.

use serde::{Deserialize, Serialize};
use systemprompt_models::ai::tools::McpTool;

/// MCP server metadata for A2A extension params
///
/// Used in `systemprompt:mcp-integration` extension to communicate
/// available MCP servers to API consumers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpServerMetadata {
    /// MCP server name (e.g., "systemprompt-admin")
    pub name: String,

    /// HTTP endpoint for MCP protocol
    pub endpoint: String,

    /// Required authentication role/scope
    pub auth: String,

    /// Runtime status (running, stopped, not_found)
    pub status: String,

    /// Server version from manifest
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Available tools from this MCP server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<McpTool>>,
}
