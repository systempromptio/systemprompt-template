use rmcp::{
    model::{CallToolRequestParam, CallToolResult, Content},
    ErrorData as McpError,
};
use std::fs;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::models::artifacts::{ExecutionMetadata, TextArtifact, ToolResponse};

use crate::error::ToolError;
use crate::SystemToolsServer;

use super::ToolArguments;

pub fn handle(
    request: CallToolRequestParam,
    server: &SystemToolsServer,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let arguments = ToolArguments::new(request.arguments);

    let file_path = arguments.get_required_path("file_path")?;
    let content = arguments.get_required_string("content")?;

    let validated_path = if file_path.exists() {
        server
            .validate_path(&file_path)
            .map_err(|error| McpError::invalid_params(error, None))?
    } else {
        server
            .validate_new_path(&file_path)
            .map_err(|error| McpError::invalid_params(error, None))?
    };

    if let Some(parent) = validated_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|error| ToolError::IoError {
                details: format!("Failed to create directories: {error}"),
            })?;
        }
    }

    let bytes_written = content.len();
    fs::write(&validated_path, content).map_err(|error| ToolError::IoError {
        details: format!("Failed to write file: {error}"),
    })?;

    let line_count = content.lines().count();

    let metadata = ExecutionMetadata::new().tool("write_file");
    let artifact_id = uuid::Uuid::new_v4().to_string();

    let artifact_content = format!(
        "Successfully wrote {bytes_written} bytes ({line_count} lines) to {}",
        validated_path.display()
    );

    let artifact = TextArtifact::new(&artifact_content)
        .with_title(format!("Write: {}", validated_path.display()));

    let tool_response = ToolResponse::new(
        &artifact_id,
        mcp_execution_id.clone(),
        artifact,
        metadata.clone(),
    );

    let summary = format!(
        "Wrote {bytes_written} bytes to {}",
        validated_path.display()
    );

    Ok(CallToolResult {
        content: vec![Content::text(summary)],
        is_error: Some(false),
        meta: metadata.to_meta(),
        structured_content: Some(tool_response.to_json()),
    })
}
