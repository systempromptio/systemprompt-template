use glob::glob as glob_match;
use rmcp::{model::*, ErrorData as McpError};
use std::path::PathBuf;

use crate::SystemToolsServer;

pub async fn handle(
    request: CallToolRequestParam,
    server: &SystemToolsServer,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();

    // Parse required pattern
    let pattern = args
        .get("pattern")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing 'pattern' parameter", None))?;

    // Parse optional base path
    let base_path = SystemToolsServer::parse_optional_string(&args, "path");

    // Determine the search directory
    let search_dir = if let Some(ref path_str) = base_path {
        let path = PathBuf::from(path_str);
        server
            .validate_path(&path)
            .await
            .map_err(|e| McpError::invalid_params(e, None))?
    } else {
        // Use first root as default
        let roots = server.get_roots().await;
        roots.first().cloned().unwrap_or_else(|| PathBuf::from("."))
    };

    if !search_dir.is_dir() {
        return Err(McpError::invalid_params(
            format!("Path is not a directory: {}", search_dir.display()),
            None,
        ));
    }

    // Construct the full glob pattern
    let full_pattern = search_dir.join(pattern);
    let pattern_str = full_pattern.to_string_lossy();

    // Execute glob
    let entries: Vec<PathBuf> = glob_match(&pattern_str)
        .map_err(|e| McpError::invalid_params(format!("Invalid glob pattern: {e}"), None))?
        .filter_map(Result::ok)
        .collect();

    // Sort by modification time (newest first)
    let mut entries_with_time: Vec<(PathBuf, std::time::SystemTime)> = entries
        .into_iter()
        .filter_map(|p| {
            p.metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| (p, t))
        })
        .collect();

    entries_with_time.sort_by(|a, b| b.1.cmp(&a.1));

    let sorted_entries: Vec<PathBuf> = entries_with_time.into_iter().map(|(p, _)| p).collect();

    // Format output
    let count = sorted_entries.len();

    if count == 0 {
        return Ok(CallToolResult {
            content: vec![Content::text(format!(
                "No files found matching pattern '{}' in {}",
                pattern,
                search_dir.display()
            ))],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        });
    }

    // Limit output to prevent overwhelming results
    let max_results = 500;
    let display_entries: Vec<&PathBuf> = sorted_entries.iter().take(max_results).collect();
    let truncated = count > max_results;

    let mut output = String::new();

    for entry in &display_entries {
        output.push_str(&format!("{}\n", entry.display()));
    }

    let header = if truncated {
        format!(
            "Found {} files matching '{}' (showing first {}):\n\n",
            count, pattern, max_results
        )
    } else {
        format!("Found {} files matching '{}':\n\n", count, pattern)
    };

    Ok(CallToolResult {
        content: vec![Content::text(format!("{header}{output}"))],
        is_error: Some(false),
        meta: None,
        structured_content: None,
    })
}
