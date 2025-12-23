use rmcp::{
    model::{CallToolRequestParam, CallToolResult, Content},
    ErrorData as McpError,
};
use std::fs;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::models::artifacts::{ExecutionMetadata, TextArtifact, ToolResponse};

use crate::constants::MAX_DISPLAY_TRUNCATE;
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
    let old_string = arguments.get_required_string("old_string")?;
    let new_string = arguments.get_required_string("new_string")?;
    let replace_all = arguments.get_bool_or("replace_all", false);

    let canonical_path = server
        .validate_path(&file_path)
        .map_err(|error| McpError::invalid_params(error, None))?;

    if !canonical_path.exists() {
        return Err(ToolError::FileNotFound {
            path: file_path.display().to_string(),
        }
        .into());
    }

    if !canonical_path.is_file() {
        return Err(ToolError::NotAFile {
            path: file_path.display().to_string(),
        }
        .into());
    }

    let content = fs::read_to_string(&canonical_path)
        .map_err(|error| McpError::internal_error(format!("Failed to read file: {error}"), None))?;

    let occurrence_count = content.matches(old_string).count();

    if occurrence_count == 0 {
        return Err(ToolError::StringNotFound {
            needle: truncate_for_display(old_string, MAX_DISPLAY_TRUNCATE),
            path: file_path.display().to_string(),
        }
        .into());
    }

    if !replace_all && occurrence_count > 1 {
        return Err(ToolError::AmbiguousEdit {
            count: occurrence_count,
        }
        .into());
    }

    let new_content = if replace_all {
        content.replace(old_string, new_string)
    } else {
        content.replacen(old_string, new_string, 1)
    };

    fs::write(&canonical_path, &new_content).map_err(|error| {
        McpError::internal_error(format!("Failed to write file: {error}"), None)
    })?;

    let replacements = if replace_all { occurrence_count } else { 1 };

    let metadata = ExecutionMetadata::new().tool("edit_file");
    let artifact_id = uuid::Uuid::new_v4().to_string();

    let artifact_content = format!(
        "Successfully edited {}: {replacements} replacement(s) made",
        canonical_path.display()
    );

    let artifact = TextArtifact::new(&artifact_content)
        .with_title(format!("Edit: {}", canonical_path.display()));

    let tool_response = ToolResponse::new(
        &artifact_id,
        mcp_execution_id.clone(),
        artifact,
        metadata.clone(),
    );

    let summary = format!(
        "Replaced {replacements} occurrence(s) in {}",
        canonical_path.display()
    );

    Ok(CallToolResult {
        content: vec![Content::text(summary)],
        is_error: Some(false),
        meta: metadata.to_meta(),
        structured_content: Some(tool_response.to_json()),
    })
}

fn truncate_for_display(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}...", &text[..max_length])
    }
}
