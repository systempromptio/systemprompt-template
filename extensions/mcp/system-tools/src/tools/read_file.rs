use rmcp::{
    model::{CallToolRequestParam, CallToolResult, Content},
    ErrorData as McpError,
};
use std::fmt::Write;
use std::fs;
use systemprompt::identifiers::{ArtifactId, McpExecutionId};
use systemprompt::models::artifacts::{ExecutionMetadata, TextArtifact, ToolResponse};

use crate::constants::{DEFAULT_LINE_LIMIT, DEFAULT_LINE_OFFSET, MAX_LINE_DISPLAY_LENGTH};
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

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let offset = arguments.get_i64_or("offset", DEFAULT_LINE_OFFSET).max(1) as usize;
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let limit = arguments.get_i64_or("limit", DEFAULT_LINE_LIMIT).max(1) as usize;

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

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let start_index = (offset - 1).min(total_lines);
    let end_index = (start_index + limit).min(total_lines);
    let lines_read = end_index - start_index;

    let mut output = String::new();

    for (index, line) in lines[start_index..end_index].iter().enumerate() {
        let line_number = start_index + index + 1;
        let display_line = if line.len() > MAX_LINE_DISPLAY_LENGTH {
            format!("{}... [truncated]", &line[..MAX_LINE_DISPLAY_LENGTH])
        } else {
            (*line).to_string()
        };
        let _ = writeln!(output, "{line_number:6}\t{display_line}");
    }

    let header = if start_index > 0 || end_index < total_lines {
        format!(
            "Showing lines {}-{} of {total_lines} total lines from {}\n\n",
            start_index + 1,
            end_index,
            canonical_path.display()
        )
    } else {
        format!(
            "File: {} ({total_lines} lines)\n\n",
            canonical_path.display(),
        )
    };

    let artifact_content = format!("{header}{output}");

    let metadata = ExecutionMetadata::new().tool("read_file");
    let artifact_id = ArtifactId::new(uuid::Uuid::new_v4().to_string());

    let artifact = TextArtifact::new(&artifact_content)
        .with_title(format!("File: {}", canonical_path.display()));

    let tool_response = ToolResponse::new(
        artifact_id,
        mcp_execution_id.clone(),
        artifact,
        metadata.clone(),
    );

    let summary = if start_index > 0 || end_index < total_lines {
        format!(
            "Read lines {}-{} of {total_lines} from {}",
            start_index + 1,
            end_index,
            canonical_path.display()
        )
    } else {
        format!("Read {lines_read} lines from {}", canonical_path.display())
    };

    Ok(CallToolResult {
        content: vec![Content::text(summary)],
        is_error: Some(false),
        meta: metadata.to_meta(),
        structured_content: Some(tool_response.to_json()),
    })
}
