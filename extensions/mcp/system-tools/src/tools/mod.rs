mod edit_file;
mod file_context;
mod glob;
mod grep;
mod list_files;
mod read_file;
mod write_file;

use rmcp::{
    model::{CallToolRequestMethod, CallToolRequestParam, CallToolResult, Tool},
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::logging::LogService;
use systemprompt::models::execution::context::RequestContext as SysRequestContext;

use crate::error::ToolError;
use crate::SystemToolsServer;

pub struct ToolArguments {
    inner: serde_json::Map<String, serde_json::Value>,
}

impl ToolArguments {
    #[must_use]
    pub fn new(arguments: Option<serde_json::Map<String, serde_json::Value>>) -> Self {
        Self {
            inner: arguments.unwrap_or_default(),
        }
    }

    pub fn get_required_string(&self, key: &str) -> Result<&str, ToolError> {
        self.inner
            .get(key)
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| ToolError::MissingParameter {
                name: key.to_string(),
            })
    }

    #[must_use]
    pub fn get_optional_string(&self, key: &str) -> Option<&str> {
        self.inner.get(key).and_then(serde_json::Value::as_str)
    }

    #[must_use]
    pub fn get_i64_or(&self, key: &str, default: i64) -> i64 {
        self.inner
            .get(key)
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(default)
    }

    #[must_use]
    pub fn get_bool_or(&self, key: &str, default: bool) -> bool {
        self.inner
            .get(key)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(default)
    }

    pub fn get_required_path(&self, key: &str) -> Result<PathBuf, ToolError> {
        self.get_required_string(key).map(PathBuf::from)
    }

    #[must_use]
    pub fn get_optional_path(&self, key: &str) -> Option<PathBuf> {
        self.get_optional_string(key).map(PathBuf::from)
    }
}

#[must_use]
pub fn register_tools() -> Vec<Tool> {
    vec![
        create_tool(
            "read_file",
            "Read File",
            "Read the contents of a file. Supports reading specific line ranges with offset and limit parameters.",
            &read_file_input_schema(),
        ),
        create_tool(
            "write_file",
            "Write File",
            "Write content to a file, creating it if it doesn't exist or overwriting if it does.",
            &write_file_input_schema(),
        ),
        create_tool(
            "edit_file",
            "Edit File",
            "Edit a file by replacing specific text. Use old_string to find text and new_string to replace it. Set replace_all to true to replace all occurrences.",
            &edit_file_input_schema(),
        ),
        create_tool(
            "glob",
            "Glob Search",
            "Find files matching a glob pattern (e.g., '**/*.rs', 'src/**/*.ts'). Returns file paths sorted by modification time.",
            &glob_input_schema(),
        ),
        create_tool(
            "grep",
            "Grep Search",
            "Search file contents using regex patterns. Can search a single file or recursively search a directory.",
            &grep_input_schema(),
        ),
        create_tool(
            "list_files",
            "List Files",
            "List files and directories in a tree structure. Shows the directory hierarchy with configurable depth. Use this to understand the file structure before reading or editing files.",
            &list_files_input_schema(),
        ),
        create_tool(
            "file_context",
            "File Context",
            "Gather comprehensive context about a codebase using AI-powered reasoning. Iteratively explores directory structure, reads relevant files, and searches for patterns to answer questions about the code.",
            &file_context::file_context_input_schema(),
        ),
    ]
}

pub async fn handle_tool_call(
    name: &str,
    request: CallToolRequestParam,
    _context: RequestContext<RoleServer>,
    logger: LogService,
    server: &SystemToolsServer,
    mcp_execution_id: &McpExecutionId,
    sys_context: SysRequestContext,
) -> Result<CallToolResult, McpError> {
    logger
        .debug("system_tools", &format!("Executing tool: {name}"))
        .await
        .ok();

    let result = match name {
        "read_file" => read_file::handle(request, server, mcp_execution_id),
        "write_file" => write_file::handle(request, server, mcp_execution_id),
        "edit_file" => edit_file::handle(request, server, mcp_execution_id),
        "glob" => glob::handle(request, server, mcp_execution_id),
        "grep" => grep::handle(request, server, mcp_execution_id),
        "list_files" => list_files::handle(request, server, mcp_execution_id),
        "file_context" => {
            return file_context::handle(request, server, &logger, mcp_execution_id, sys_context)
                .await;
        }
        _ => {
            logger
                .warn("system_tools", &format!("Unknown tool: {name}"))
                .await
                .ok();
            return Err(McpError::method_not_found::<CallToolRequestMethod>());
        }
    };

    match &result {
        Ok(_) => {
            logger
                .debug(
                    "system_tools",
                    &format!("Tool {name} completed successfully"),
                )
                .await
                .ok();
        }
        Err(error) => {
            logger
                .error("system_tools", &format!("Tool {name} failed: {error:?}"))
                .await
                .ok();
        }
    }

    result
}

fn create_tool(
    name: &str,
    title: &str,
    description: &str,
    input_schema: &serde_json::Value,
) -> Tool {
    let input_object = input_schema.as_object().cloned().unwrap_or_default();

    Tool {
        name: name.to_string().into(),
        title: Some(title.to_string()),
        description: Some(description.to_string().into()),
        input_schema: Arc::new(input_object),
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

fn list_files_input_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "path": {
                "type": "string",
                "description": "Directory path to list (default: root directory)"
            },
            "depth": {
                "type": "integer",
                "description": "Maximum depth to traverse (default: 3, max: 5)"
            }
        },
        "required": []
    })
}
