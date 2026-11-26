//! Persistence for MCP server configuration and runtime state.

pub mod mcp_servers;
#[cfg(feature = "mcp")]
pub mod runtime;
