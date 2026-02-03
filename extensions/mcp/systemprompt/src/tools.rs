use rmcp::model::{Meta, Tool};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    Table,
    List,
    PresentationCard,
    Text,
    CopyPasteText,
    Chart,
    Form,
    Dashboard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingHints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub data: serde_json::Value,
    pub artifact_type: ArtifactType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<RenderingHints>,
}

impl CommandResult {
    #[must_use]
    pub fn from_stdout(stdout: &str) -> Option<Self> {
        serde_json::from_str(stdout).ok()
    }

    #[must_use]
    pub fn artifact_type_str(&self) -> &'static str {
        match self.artifact_type {
            ArtifactType::Table => "table",
            ArtifactType::List => "list",
            ArtifactType::PresentationCard => "presentation_card",
            ArtifactType::Text => "text",
            ArtifactType::CopyPasteText => "copy_paste_text",
            ArtifactType::Chart => "chart",
            ArtifactType::Form => "form",
            ArtifactType::Dashboard => "dashboard",
        }
    }
}

#[must_use]
pub fn create_result_meta(artifact_id: &str) -> Meta {
    let mut meta_map = serde_json::Map::new();
    meta_map.insert(
        "ui".to_string(),
        serde_json::json!({
            "resourceUri": format!("ui://{}/{}", SERVER_NAME, artifact_id),
            "visibility": ["model"]
        }),
    );
    Meta(meta_map)
}

fn create_ui_meta() -> Meta {
    let mut meta_map = serde_json::Map::new();
    meta_map.insert(
        "ui".to_string(),
        serde_json::json!({
            "resourceUri": format!("ui://{}/{{artifact_id}}", SERVER_NAME),
            "visibility": ["model"]
        }),
    );
    Meta(meta_map)
}

#[must_use]
pub fn input_schema() -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    map.insert("type".to_string(), serde_json::json!("object"));
    map.insert("properties".to_string(), serde_json::json!({
        "command": {
            "type": "string",
            "description": "FIRST COMMAND: 'core playbooks show guide_start' - MANDATORY before any task. Then use playbook commands. Do not include 'systemprompt' prefix."
        }
    }));
    map.insert("required".to_string(), serde_json::json!(["command"]));
    map
}

#[must_use]
pub fn output_schema() -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    map.insert("type".to_string(), serde_json::json!("object"));
    map.insert(
        "properties".to_string(),
        serde_json::json!({
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
        }),
    );
    map.insert(
        "required".to_string(),
        serde_json::json!(["stdout", "stderr", "exit_code", "success"]),
    );
    map
}

#[must_use]
pub fn list_tools() -> Vec<Tool> {
    vec![create_cli_tool()]
}

fn create_cli_tool() -> Tool {
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
        input_schema: Arc::new(input_schema()),
        output_schema: Some(Arc::new(output_schema())),
        annotations: None,
        icons: None,
        meta: Some(create_ui_meta()),
    }
}
