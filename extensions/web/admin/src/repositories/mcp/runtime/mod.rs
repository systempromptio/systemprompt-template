//! Runtime state for the MCP servers declared in `services/mcp/*.yaml`.
//!
//! The YAML says what a server *is*; these queries say what it has been
//! *doing*. Three tables carry that: `mcp_sessions` (a client attached, and
//! when it last spoke), `mcp_proxy_identities` (the identity a live session is
//! acting as), and `mcp_tool_executions` (every tool call, with its outcome).
//!
//! Neither the heartbeat nor the rule that reads it lives here:
//! `repositories::overview::liveness` owns both, because the dashboard shows
//! the same fact and one answer for both is the point.
//!
//! Every function here is keyed by the server name as the runtime records it,
//! which is not guaranteed to be a name the catalog declares. A server that
//! appears only in the executions table is a real thing that really ran, so it
//! is returned rather than filtered out; the pages surface it as unconfigured.

mod executions;
mod liveness;

pub use executions::{
    McpExecutionRow, McpServerActivity, McpToolStat, list_mcp_executions_paged,
    list_mcp_server_activity, list_mcp_tool_stats,
};
pub use liveness::{
    McpProxyIdentityCount, McpSessionRow, list_mcp_proxy_identity_counts,
    list_mcp_sessions_for_server,
};
