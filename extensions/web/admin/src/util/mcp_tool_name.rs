//! The server and bare tool behind a host's namespaced MCP tool name.
//!
//! Claude Code presents an MCP tool as `mcp__<server>__<tool>`, and a server
//! installed through a plugin as `mcp__plugin_<marketplace>_<server>__<tool>`.
//! The gateway records the in-process execution under the bare tool and
//! server names, so a hook report has to be reduced to the same two names
//! before the ingest can recognise both as one call. Server ids may contain
//! `_` (`google_workspace`), so the plugin form is resolved against the servers
//! this instance declares and falls back to the last `_` only when none
//! matches.

use systemprompt::loader::ServicesBootstrap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpToolName {
    pub server: String,
    pub tool: String,
}

// Why: resolves against the servers this instance declares.
#[must_use]
pub fn parse_mcp_tool_name(tool_name: &str) -> Option<McpToolName> {
    let known: Vec<String> = ServicesBootstrap::get().map_or_else(
        |_| Vec::new(),
        |services| services.mcp_servers.keys().cloned().collect(),
    );
    parse_mcp_tool_name_with(tool_name, &known)
}

// Why: the pure form — `known` is the set of server ids a plugin namespace
// may end in — so a test needs no services tree.
#[must_use]
pub fn parse_mcp_tool_name_with(tool_name: &str, known: &[String]) -> Option<McpToolName> {
    let rest = tool_name.strip_prefix("mcp__")?;
    let (namespace, tool) = rest.rsplit_once("__")?;
    if namespace.is_empty() || tool.is_empty() {
        return None;
    }
    let server = namespace.strip_prefix("plugin_").map_or_else(
        || namespace.to_owned(),
        |scoped| plugin_server(scoped, known),
    );
    Some(McpToolName {
        server,
        tool: tool.to_owned(),
    })
}

fn plugin_server(scoped: &str, known: &[String]) -> String {
    known
        .iter()
        .filter(|id| scoped.ends_with(&format!("_{id}")))
        .max_by_key(|id| id.len())
        .cloned()
        .unwrap_or_else(|| {
            scoped
                .rsplit_once('_')
                .map_or(scoped, |(_, server)| server)
                .to_owned()
        })
}
