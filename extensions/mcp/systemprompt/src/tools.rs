use rmcp::model::{Meta, Tool};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::mcp::{default_tool_visibility, tool_ui_meta};
use systemprompt::models::artifacts::{CliArtifact, ToolResponse};

pub const SERVER_NAME: &str = "systemprompt";

#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn create_ui_meta() -> Meta {
    Meta(tool_ui_meta(SERVER_NAME, &default_tool_visibility()))
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

#[must_use]
pub fn list_tools() -> Vec<Tool> {
    vec![create_cli_tool()]
}

fn create_cli_tool() -> Tool {
    let input = input_schema();
    let output = output_schema();

    let input_obj = input
        .as_object()
        .cloned()
        .expect("input_schema must be a JSON object");
    let output_obj = output
        .as_object()
        .cloned()
        .expect("output_schema must be a JSON object");

    Tool {
        name: "systemprompt".to_string().into(),
        title: Some("SystemPrompt CLI".to_string()),
        description: Some(
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
            Full documentation: https://systemprompt.io/playbooks"
                .to_string()
                .into(),
        ),
        input_schema: Arc::new(input_obj),
        output_schema: Some(Arc::new(output_obj)),
        annotations: None,
        icons: None,
        meta: Some(create_ui_meta()),
    }
}
