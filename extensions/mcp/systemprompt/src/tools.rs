//! Tool definitions exposed by the `systemprompt` MCP server.

use rmcp::model::{MetaObject, Tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::mcp::{McpOutputSchema, WEBSITE_URL, default_tool_visibility, tool_ui_meta};
use systemprompt::models::artifacts::CliArtifact;

pub const SERVER_NAME: &str = "systemprompt";

// Why: a const rather than a literal at the call site so the wire name is
// extractable from source. `scripts/check-mcp-tool-names.sh` builds its
// catalog from these declarations — a tool that only names itself inline is
// a tool the gate cannot vouch for.
pub const TOOL_SYSTEMPROMPT: &str = "systemprompt";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CliInput {
    /// The CLI command to execute (without 'systemprompt' prefix). Examples:
    /// 'plugins run discord send "message"', 'core skills list'
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

#[must_use]
// JSON: protocol boundary
pub fn input_schema() -> serde_json::Value {
    schemars::schema_for!(CliInput).to_value()
}

#[must_use]
// JSON: protocol boundary
pub fn output_schema() -> serde_json::Value {
    <CliArtifact as McpOutputSchema>::validated_schema()
}

struct ToolDef<'a> {
    server_name: &'a str,
    name: &'a str,
    title: &'a str,
    description: &'a str,
    // JSON: protocol boundary
    input_schema: &'a serde_json::Value,
    // JSON: protocol boundary
    output_schema: &'a serde_json::Value,
}

fn create_tool(def: &ToolDef<'_>) -> Tool {
    let input_obj = def
        .input_schema
        .as_object()
        .cloned()
        .unwrap_or_else(serde_json::Map::new);
    let output_obj = def
        .output_schema
        .as_object()
        .cloned()
        .unwrap_or_else(serde_json::Map::new);

    let mut tool = Tool::default();
    tool.name = def.name.to_owned().into();
    tool.title = Some(def.title.to_owned());
    tool.description = Some(def.description.to_owned().into());
    tool.input_schema = Arc::new(input_obj);
    tool.output_schema = Some(Arc::new(output_obj));
    tool.meta = Some(MetaObject(tool_ui_meta(
        def.server_name,
        &default_tool_visibility(),
    )));
    tool
}

#[must_use]
pub fn list_tools() -> Vec<Tool> {
    let desc = format!(
        "Execute SystemPrompt CLI commands. Pass the command WITHOUT the 'systemprompt' prefix.\n\n\
        Common commands:\n  \
        - core skills list: List installed skills\n  \
        - core skills show <id>: Show a skill's config and instruction body\n  \
        - core content list: List markdown content\n  \
        - plugins run discord send \"message\": Send Discord notification\n  \
        - plugins run discord send \"message\" --channel <id>: Send to specific channel\n  \
        - admin agents list: List agents\n\n\
        Example: {{\"command\": \"core skills list\"}}\n\n\
        Full documentation: {WEBSITE_URL}/docs"
    );
    vec![create_tool(&ToolDef {
        server_name: SERVER_NAME,
        name: TOOL_SYSTEMPROMPT,
        title: "SystemPrompt CLI",
        description: &desc,
        input_schema: &input_schema(),
        output_schema: &output_schema(),
    })]
}
