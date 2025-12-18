use rmcp::{model::*, ErrorData as McpError};
use std::fs;

use crate::SystemToolsServer;

pub async fn handle(
    request: CallToolRequestParam,
    server: &SystemToolsServer,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();

    // Parse required file_path
    let file_path = SystemToolsServer::parse_file_path(&args, "file_path")?;

    // Parse optional parameters
    let offset = SystemToolsServer::parse_optional_i64(&args, "offset")
        .map(|v| v.max(1) as usize)
        .unwrap_or(1);
    let limit = SystemToolsServer::parse_optional_i64(&args, "limit")
        .map(|v| v.max(1) as usize)
        .unwrap_or(2000);

    // Validate path is within allowed roots
    let canonical_path = server
        .validate_path(&file_path)
        .await
        .map_err(|e| McpError::invalid_params(e, None))?;

    // Check if file exists
    if !canonical_path.exists() {
        return Err(McpError::invalid_params(
            format!("File does not exist: {}", file_path.display()),
            None,
        ));
    }

    if !canonical_path.is_file() {
        return Err(McpError::invalid_params(
            format!("Path is not a file: {}", file_path.display()),
            None,
        ));
    }

    // Read the file
    let content = fs::read_to_string(&canonical_path)
        .map_err(|e| McpError::internal_error(format!("Failed to read file: {e}"), None))?;

    // Apply line offset and limit (1-based indexing)
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let start_idx = (offset - 1).min(total_lines);
    let end_idx = (start_idx + limit).min(total_lines);

    // Format output with line numbers (like cat -n)
    let mut output = String::new();

    for (idx, line) in lines[start_idx..end_idx].iter().enumerate() {
        let line_num = start_idx + idx + 1;
        // Truncate long lines
        let display_line = if line.len() > 2000 {
            format!("{}... [truncated]", &line[..2000])
        } else {
            line.to_string()
        };
        output.push_str(&format!("{:6}\t{}\n", line_num, display_line));
    }

    // Add metadata about the read
    let header = if start_idx > 0 || end_idx < total_lines {
        format!(
            "Showing lines {}-{} of {} total lines from {}\n\n",
            start_idx + 1,
            end_idx,
            total_lines,
            canonical_path.display()
        )
    } else {
        format!(
            "File: {} ({} lines)\n\n",
            canonical_path.display(),
            total_lines
        )
    };

    Ok(CallToolResult {
        content: vec![Content::text(format!("{header}{output}"))],
        is_error: Some(false),
        meta: None,
        structured_content: None,
    })
}
