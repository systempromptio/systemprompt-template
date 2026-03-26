use rmcp::model::{Meta, Tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::mcp::{default_tool_visibility, tool_ui_meta};
use systemprompt::models::artifacts::{CliArtifact, ToolResponse};

pub const SERVER_NAME: &str = "systemprompt";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CliInput {
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
pub fn input_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "command": {
                "type": "string",
                "description": "The CLI command to execute (without 'systemprompt' prefix). Examples: 'plugins run discord send \"message\"', 'core playbooks list'"
            }
        },
        "required": ["command"]
    })
}

#[must_use]
pub fn output_schema() -> serde_json::Value {
    ToolResponse::<CliArtifact>::schema()
}

fn create_tool(
    server_name: &str,
    name: &str,
    title: &str,
    description: &str,
    input_schema: &serde_json::Value,
    output_schema: &serde_json::Value,
) -> Tool {
    let input_obj = input_schema.as_object().cloned().unwrap_or_default();
    let output_obj = output_schema.as_object().cloned().unwrap_or_default();

    let mut tool = Tool::default();
    tool.name = name.to_string().into();
    tool.title = Some(title.to_string());
    tool.description = Some(description.to_string().into());
    tool.input_schema = Arc::new(input_obj);
    tool.output_schema = Some(Arc::new(output_obj));
    tool.meta = Some(Meta(tool_ui_meta(server_name, &default_tool_visibility())));
    tool
}

#[must_use]
pub fn list_tools() -> Vec<Tool> {
    vec![create_tool(
        SERVER_NAME,
        "systemprompt",
        "SystemPrompt CLI",
        "Execute SystemPrompt CLI commands. Pass the command WITHOUT the 'systemprompt' prefix.\n\n\
        MANDATORY FIRST STEP: Run 'core playbooks show guide_start' before any task.\n\n\
        Common commands:\n  \
        - core playbooks show guide_start: Load the getting started guide (ALWAYS DO THIS FIRST)\n  \
        - core playbooks show <id>: Load a playbook\n  \
        - core playbooks list: List playbooks\n  \
        - plugins run discord send \"message\": Send Discord notification\n  \
        - plugins run discord send \"message\" --channel <id>: Send to specific channel\n  \
        - admin agents list: List agents\n\n\
        Example: {\"command\": \"core playbooks show guide_start\"}\n\n\
        Full documentation: https://foodles.com/playbooks",
        &input_schema(),
        &output_schema(),
    )]
}
