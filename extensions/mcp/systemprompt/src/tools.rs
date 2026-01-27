use rmcp::model::Tool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

#[must_use]
pub fn input_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "command": {
                "type": "string",
                "description": "FIRST COMMAND: 'core playbooks show guide_start' - MANDATORY before any task. Then use playbook commands. Do not include 'systemprompt' prefix."
            }
        },
        "required": ["command"]
    })
}

#[must_use]
pub fn output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "stdout": {
                "type": "string",
                "description": "Standard output from the command"
            },
            "stderr": {
                "type": "string",
                "description": "Standard error output from the command"
            },
            "exit_code": {
                "type": "integer",
                "description": "Exit code of the command (0 = success)"
            },
            "success": {
                "type": "boolean",
                "description": "Whether the command succeeded"
            }
        },
        "required": ["stdout", "stderr", "exit_code", "success"]
    })
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
            "Execute SystemPrompt CLI commands.\n\n\
            *** MANDATORY FIRST STEP ***\n\
            Run 'core playbooks show guide_start' BEFORE any other command.\n\
            This loads the required playbook guide that explains how to use this system.\n\n\
            Agents MUST:\n\
            1. Load guide_start playbook first\n\
            2. Follow playbook instructions exactly\n\
            3. Never improvise commands\n\n\
            Commands:\n  \
            - core playbooks show guide_start: REQUIRED FIRST (load playbook guide)\n  \
            - core playbooks list: List all playbooks\n  \
            - core playbooks show <id>: Load a specific playbook"
                .to_string()
                .into(),
        ),
        input_schema: Arc::new(input_obj),
        output_schema: Some(Arc::new(output_obj)),
        annotations: None,
        icons: None,
        meta: None,
    }
}
