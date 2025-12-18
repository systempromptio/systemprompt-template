use rmcp::{model::*, ErrorData as McpError};
use std::fs;

use crate::SystemToolsServer;

pub async fn handle(
    request: CallToolRequestParam,
    server: &SystemToolsServer,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();

    // Parse required parameters
    let file_path = SystemToolsServer::parse_file_path(&args, "file_path")?;

    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing 'content' parameter", None))?;

    // Validate path - for new files, check parent directory
    let validated_path = if file_path.exists() {
        server
            .validate_path(&file_path)
            .await
            .map_err(|e| McpError::invalid_params(e, None))?
    } else {
        server
            .validate_parent_path(&file_path)
            .await
            .map_err(|e| McpError::invalid_params(e, None))?
    };

    // Create parent directories if they don't exist
    if let Some(parent) = validated_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                McpError::internal_error(format!("Failed to create directories: {e}"), None)
            })?;
        }
    }

    // Write the file
    let bytes_written = content.len();
    fs::write(&validated_path, content)
        .map_err(|e| McpError::internal_error(format!("Failed to write file: {e}"), None))?;

    let line_count = content.lines().count();

    Ok(CallToolResult {
        content: vec![Content::text(format!(
            "Successfully wrote {} bytes ({} lines) to {}",
            bytes_written,
            line_count,
            validated_path.display()
        ))],
        is_error: Some(false),
        meta: None,
        structured_content: None,
    })
}
