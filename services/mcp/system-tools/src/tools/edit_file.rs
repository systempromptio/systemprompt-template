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

    let old_string = args
        .get("old_string")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing 'old_string' parameter", None))?;

    let new_string = args
        .get("new_string")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing 'new_string' parameter", None))?;

    let replace_all = SystemToolsServer::parse_optional_bool(&args, "replace_all").unwrap_or(false);

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

    // Check if old_string exists in the file
    let occurrence_count = content.matches(old_string).count();

    if occurrence_count == 0 {
        return Err(McpError::invalid_params(
            format!(
                "String not found in file. The old_string '{}' does not exist in {}",
                truncate_for_display(old_string, 100),
                file_path.display()
            ),
            None,
        ));
    }

    // Check for ambiguous replacement (multiple occurrences when not using replace_all)
    if !replace_all && occurrence_count > 1 {
        return Err(McpError::invalid_params(
            format!(
                "Ambiguous edit: found {} occurrences of old_string. Use replace_all=true to replace all, or provide a more specific old_string.",
                occurrence_count
            ),
            None,
        ));
    }

    // Perform the replacement
    let new_content = if replace_all {
        content.replace(old_string, new_string)
    } else {
        content.replacen(old_string, new_string, 1)
    };

    // Write the modified content back
    fs::write(&canonical_path, &new_content)
        .map_err(|e| McpError::internal_error(format!("Failed to write file: {e}"), None))?;

    let replacements = if replace_all { occurrence_count } else { 1 };

    Ok(CallToolResult {
        content: vec![Content::text(format!(
            "Successfully edited {}: {} replacement(s) made",
            canonical_path.display(),
            replacements
        ))],
        is_error: Some(false),
        meta: None,
        structured_content: None,
    })
}

fn truncate_for_display(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
