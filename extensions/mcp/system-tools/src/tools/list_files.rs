use rmcp::{
    model::{CallToolRequestParam, CallToolResult, Content},
    ErrorData as McpError,
};
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use systemprompt::identifiers::{ArtifactId, McpExecutionId};
use systemprompt::models::artifacts::{ExecutionMetadata, TextArtifact, ToolResponse};

use crate::constants::MAX_LIST_FILES_DEPTH;
use crate::error::ToolError;
use crate::SystemToolsServer;

use super::ToolArguments;

pub fn handle(
    request: CallToolRequestParam,
    server: &SystemToolsServer,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let arguments = ToolArguments::new(request.arguments);

    let path = arguments.get_optional_path("path");
    let max_depth = arguments.get_i64_or("depth", 3) as usize;
    let depth = max_depth.min(MAX_LIST_FILES_DEPTH);

    let target_path = match path {
        Some(p) => server
            .validate_path(&p)
            .map_err(|error| McpError::invalid_params(error, None))?,
        None => server
            .get_roots()
            .first()
            .cloned()
            .ok_or(ToolError::NoFileRoots)?,
    };

    if !target_path.is_dir() {
        return Err(ToolError::NotADirectory {
            path: target_path.display().to_string(),
        }
        .into());
    }

    let mut output = String::new();
    let mut file_count = 0;
    let mut dir_count = 0;

    build_tree(
        &target_path,
        &mut output,
        "",
        0,
        depth,
        &mut file_count,
        &mut dir_count,
    )?;

    let header = format!(
        "Directory listing for: {}\nDepth: {} | Directories: {} | Files: {}\n\n",
        target_path.display(),
        depth,
        dir_count,
        file_count
    );

    let artifact_content = format!("{header}{output}");

    let metadata = ExecutionMetadata::new().tool("list_files");
    let artifact_id = ArtifactId::new(uuid::Uuid::new_v4().to_string());
    let artifact = TextArtifact::new(&artifact_content).with_title("Directory Listing");

    let tool_response = ToolResponse::new(
        artifact_id,
        mcp_execution_id.clone(),
        artifact,
        metadata.clone(),
    );

    let summary = format!(
        "Listed {} directories and {} files in {}",
        dir_count,
        file_count,
        target_path.display()
    );

    Ok(CallToolResult {
        content: vec![Content::text(summary)],
        is_error: Some(false),
        meta: metadata.to_meta(),
        structured_content: Some(tool_response.to_json()),
    })
}

fn build_tree(
    path: &PathBuf,
    output: &mut String,
    prefix: &str,
    current_depth: usize,
    max_depth: usize,
    file_count: &mut usize,
    dir_count: &mut usize,
) -> Result<(), ToolError> {
    if current_depth > max_depth {
        return Ok(());
    }

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return Ok(()), // Skip directories we can't read
    };

    let mut items: Vec<_> = entries.filter_map(Result::ok).collect();

    // Sort: directories first, then files, both alphabetically
    items.sort_by(|a, b| {
        let a_is_dir = a.path().is_dir();
        let b_is_dir = b.path().is_dir();
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    let total = items.len();

    for (index, entry) in items.iter().enumerate() {
        let is_last = index == total - 1;
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        // Skip hidden files/directories (starting with .)
        if name.starts_with('.') {
            continue;
        }

        let connector = if is_last { "└── " } else { "├── " };
        let is_dir = entry_path.is_dir();

        if is_dir {
            *dir_count += 1;
            let _ = writeln!(output, "{prefix}{connector}{name}/");

            if current_depth < max_depth {
                let new_prefix = if is_last {
                    format!("{prefix}    ")
                } else {
                    format!("{prefix}│   ")
                };
                build_tree(
                    &entry_path,
                    output,
                    &new_prefix,
                    current_depth + 1,
                    max_depth,
                    file_count,
                    dir_count,
                )?;
            }
        } else {
            *file_count += 1;
            let _ = writeln!(output, "{prefix}{connector}{name}");
        }
    }

    Ok(())
}
