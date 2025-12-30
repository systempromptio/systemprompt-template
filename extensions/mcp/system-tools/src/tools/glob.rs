use glob::glob as glob_match;
use rmcp::{
    model::{CallToolRequestParam, CallToolResult, Content},
    ErrorData as McpError,
};
use std::fmt::Write;
use std::path::PathBuf;
use systemprompt::identifiers::{ArtifactId, McpExecutionId};
use systemprompt::models::artifacts::{ExecutionMetadata, TextArtifact, ToolResponse};

use crate::constants::MAX_GLOB_RESULTS;
use crate::error::ToolError;
use crate::SystemToolsServer;

use super::ToolArguments;

pub fn handle(
    request: CallToolRequestParam,
    server: &SystemToolsServer,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let arguments = ToolArguments::new(request.arguments);

    let pattern = arguments.get_required_string("pattern")?;
    let base_path = arguments.get_optional_path("path");

    let search_directory = match base_path {
        Some(path) => server
            .validate_path(&path)
            .map_err(|error| McpError::invalid_params(error, None))?,
        None => server
            .get_roots()
            .first()
            .cloned()
            .ok_or(ToolError::NoFileRoots)?,
    };

    if !search_directory.is_dir() {
        return Err(ToolError::NotADirectory {
            path: search_directory.display().to_string(),
        }
        .into());
    }

    let full_pattern = search_directory.join(pattern);
    let pattern_string = full_pattern.to_string_lossy();

    let entries: Vec<PathBuf> = glob_match(&pattern_string)
        .map_err(|error| ToolError::InvalidGlobPattern {
            details: error.to_string(),
        })?
        .filter_map(Result::ok)
        .collect();

    let mut entries_with_time: Vec<(PathBuf, std::time::SystemTime)> = entries
        .into_iter()
        .filter_map(|path| {
            path.metadata()
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .map(|time| (path, time))
        })
        .collect();

    entries_with_time.sort_by(|first, second| second.1.cmp(&first.1));

    let sorted_entries: Vec<PathBuf> = entries_with_time
        .into_iter()
        .map(|(path, _)| path)
        .collect();

    let count = sorted_entries.len();

    let metadata = ExecutionMetadata::new().tool("glob");
    let artifact_id = ArtifactId::new(uuid::Uuid::new_v4().to_string());

    if count == 0 {
        let artifact_content = format!(
            "No files found matching pattern '{pattern}' in {}",
            search_directory.display()
        );

        let artifact = TextArtifact::new(&artifact_content).with_title("Glob Search Results");

        let tool_response = ToolResponse::new(
            artifact_id.clone(),
            mcp_execution_id.clone(),
            artifact,
            metadata.clone(),
        );

        return Ok(CallToolResult {
            content: vec![Content::text(format!("Found 0 files matching '{pattern}'"))],
            is_error: Some(false),
            meta: metadata.to_meta(),
            structured_content: Some(tool_response.to_json()),
        });
    }

    let display_entries: Vec<&PathBuf> = sorted_entries.iter().take(MAX_GLOB_RESULTS).collect();
    let truncated = count > MAX_GLOB_RESULTS;

    let mut output = String::new();

    for entry in &display_entries {
        let _ = writeln!(output, "{}", entry.display());
    }

    let header = if truncated {
        format!("Found {count} files matching '{pattern}' (showing first {MAX_GLOB_RESULTS}):\n\n",)
    } else {
        format!("Found {count} files matching '{pattern}':\n\n")
    };

    let artifact_content = format!("{header}{output}");

    let artifact = TextArtifact::new(&artifact_content).with_title("Glob Search Results");

    let tool_response = ToolResponse::new(
        artifact_id.clone(),
        mcp_execution_id.clone(),
        artifact,
        metadata.clone(),
    );

    let summary = if truncated {
        format!("Found {count} files matching '{pattern}' (showing first {MAX_GLOB_RESULTS})")
    } else {
        format!("Found {count} files matching '{pattern}'")
    };

    Ok(CallToolResult {
        content: vec![Content::text(summary)],
        is_error: Some(false),
        meta: metadata.to_meta(),
        structured_content: Some(tool_response.to_json()),
    })
}
