mod read_file;
mod write_file;
mod edit_file;
mod glob;
mod grep;

use rmcp::{model::*, ErrorData as McpError};
use serde_json::json;
use std::sync::Arc;

use crate::SystemToolsServer;

pub fn register_tools() -> Vec<Tool> {
    vec![
        create_tool(
            "read_file",
            "Read File",
            "Read the contents of a file. Supports reading specific line ranges with offset and limit parameters.",
            read_file_input_schema(),
        ),
        create_tool(
            "write_file",
            "Write File",
            "Write content to a file, creating it if it doesn't exist or overwriting if it does.",
            write_file_input_schema(),
        ),
        create_tool(
            "edit_file",
            "Edit File",
            "Edit a file by replacing specific text. Use old_string to find text and new_string to replace it. Set replace_all to true to replace all occurrences.",
            edit_file_input_schema(),
        ),
        create_tool(
            "glob",
            "Glob Search",
            "Find files matching a glob pattern (e.g., '**/*.rs', 'src/**/*.ts'). Returns file paths sorted by modification time.",
            glob_input_schema(),
        ),
        create_tool(
            "grep",
            "Grep Search",
            "Search file contents using regex patterns. Can search a single file or recursively search a directory.",
            grep_input_schema(),
        ),
    ]
}

pub async fn handle_tool_call(
    name: &str,
    request: CallToolRequestParam,
    server: &SystemToolsServer,
) -> Result<CallToolResult, McpError> {
    match name {
        "read_file" => read_file::handle(request, server).await,
        "write_file" => write_file::handle(request, server).await,
        "edit_file" => edit_file::handle(request, server).await,
        "glob" => glob::handle(request, server).await,
        "grep" => grep::handle(request, server).await,
        _ => Err(McpError::method_not_found::<CallToolRequestMethod>()),
    }
}

fn create_tool(name: &str, title: &str, description: &str, input_schema: serde_json::Value) -> Tool {
    let input_obj = input_schema.as_object().cloned().unwrap_or_default();

    Tool {
        name: name.to_string().into(),
        title: Some(title.to_string()),
        description: Some(description.to_string().into()),
        input_schema: Arc::new(input_obj),
        output_schema: None,
        annotations: None,
        icons: None,
    }
}

fn read_file_input_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "file_path": {
                "type": "string",
                "description": "Absolute path to the file to read"
            },
            "offset": {
                "type": "integer",
                "description": "Line number to start reading from (1-based, default: 1)"
            },
            "limit": {
                "type": "integer",
                "description": "Maximum number of lines to read (default: 2000)"
            }
        },
        "required": ["file_path"]
    })
}

fn write_file_input_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "file_path": {
                "type": "string",
                "description": "Absolute path to the file to write"
            },
            "content": {
                "type": "string",
                "description": "Content to write to the file"
            }
        },
        "required": ["file_path", "content"]
    })
}

fn edit_file_input_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "file_path": {
                "type": "string",
                "description": "Absolute path to the file to edit"
            },
            "old_string": {
                "type": "string",
                "description": "The text to find and replace"
            },
            "new_string": {
                "type": "string",
                "description": "The text to replace with"
            },
            "replace_all": {
                "type": "boolean",
                "description": "Replace all occurrences (default: false, replaces first occurrence only)"
            }
        },
        "required": ["file_path", "old_string", "new_string"]
    })
}

fn glob_input_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "pattern": {
                "type": "string",
                "description": "Glob pattern to match files (e.g., '**/*.rs', 'src/**/*.ts')"
            },
            "path": {
                "type": "string",
                "description": "Base directory to search in (default: current root)"
            }
        },
        "required": ["pattern"]
    })
}

fn grep_input_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "pattern": {
                "type": "string",
                "description": "Regex pattern to search for in file contents"
            },
            "path": {
                "type": "string",
                "description": "File or directory to search in"
            },
            "glob": {
                "type": "string",
                "description": "Glob pattern to filter files (e.g., '*.rs', '*.{ts,tsx}')"
            },
            "case_insensitive": {
                "type": "boolean",
                "description": "Perform case-insensitive search (default: false)"
            }
        },
        "required": ["pattern"]
    })
}
