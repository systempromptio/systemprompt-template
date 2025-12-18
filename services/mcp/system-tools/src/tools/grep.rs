use regex::RegexBuilder;
use rmcp::{model::*, ErrorData as McpError};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

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

    // Parse optional parameters
    let search_path = SystemToolsServer::parse_optional_string(&args, "path");
    let file_glob = SystemToolsServer::parse_optional_string(&args, "glob");
    let case_insensitive =
        SystemToolsServer::parse_optional_bool(&args, "case_insensitive").unwrap_or(false);

    // Determine the search path
    let base_path = if let Some(ref path_str) = search_path {
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

    // Compile regex
    let regex = RegexBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .build()
        .map_err(|e| McpError::invalid_params(format!("Invalid regex pattern: {e}"), None))?;

    // Compile glob pattern if provided
    let glob_pattern = file_glob
        .as_ref()
        .map(|g| glob::Pattern::new(g))
        .transpose()
        .map_err(|e| McpError::invalid_params(format!("Invalid glob pattern: {e}"), None))?;

    let mut matches: Vec<GrepMatch> = Vec::new();
    let max_matches = 1000;
    let max_files = 10000;
    let mut files_searched = 0;

    // Search files
    if base_path.is_file() {
        // Search single file
        if let Some(file_matches) = search_file(&base_path, &regex, max_matches) {
            matches.extend(file_matches);
        }
    } else if base_path.is_dir() {
        // Recursively search directory
        for entry in WalkDir::new(&base_path)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            if files_searched >= max_files {
                break;
            }

            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            // Skip binary files and hidden files
            if is_likely_binary(path) || is_hidden(path) {
                continue;
            }

            // Apply glob filter if provided
            if let Some(ref glob_pat) = glob_pattern {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if !glob_pat.matches(file_name) {
                        continue;
                    }
                }
            }

            files_searched += 1;

            if let Some(file_matches) = search_file(path, &regex, max_matches - matches.len()) {
                matches.extend(file_matches);

                if matches.len() >= max_matches {
                    break;
                }
            }
        }
    } else {
        return Err(McpError::invalid_params(
            format!("Path does not exist: {}", base_path.display()),
            None,
        ));
    }

    // Format output
    if matches.is_empty() {
        return Ok(CallToolResult {
            content: vec![Content::text(format!(
                "No matches found for pattern '{}' in {} ({} files searched)",
                pattern,
                base_path.display(),
                files_searched
            ))],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        });
    }

    let truncated = matches.len() >= max_matches;
    let mut output = String::new();

    // Group by file
    let mut current_file: Option<&PathBuf> = None;

    for m in &matches {
        if current_file != Some(&m.file) {
            if current_file.is_some() {
                output.push('\n');
            }
            output.push_str(&format!("{}:\n", m.file.display()));
            current_file = Some(&m.file);
        }
        output.push_str(&format!("  {:6}: {}\n", m.line_num, m.line.trim()));
    }

    let header = if truncated {
        format!(
            "Found {} matches for '{}' (truncated, showing first {}):\n\n",
            matches.len(),
            pattern,
            max_matches
        )
    } else {
        format!(
            "Found {} matches for '{}' in {} files:\n\n",
            matches.len(),
            pattern,
            files_searched
        )
    };

    Ok(CallToolResult {
        content: vec![Content::text(format!("{header}{output}"))],
        is_error: Some(false),
        meta: None,
        structured_content: None,
    })
}

struct GrepMatch {
    file: PathBuf,
    line_num: usize,
    line: String,
}

fn search_file(path: &std::path::Path, regex: &regex::Regex, max_matches: usize) -> Option<Vec<GrepMatch>> {
    let content = fs::read_to_string(path).ok()?;

    let mut matches = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        if regex.is_match(line) {
            matches.push(GrepMatch {
                file: path.to_path_buf(),
                line_num: line_num + 1,
                line: truncate_line(line, 200),
            });

            if matches.len() >= max_matches {
                break;
            }
        }
    }

    if matches.is_empty() {
        None
    } else {
        Some(matches)
    }
}

fn truncate_line(line: &str, max_len: usize) -> String {
    if line.len() <= max_len {
        line.to_string()
    } else {
        format!("{}...", &line[..max_len])
    }
}

fn is_likely_binary(path: &std::path::Path) -> bool {
    let binary_extensions = [
        "exe", "dll", "so", "dylib", "bin", "obj", "o", "a", "lib",
        "png", "jpg", "jpeg", "gif", "bmp", "ico", "webp", "svg",
        "mp3", "mp4", "avi", "mov", "mkv", "flac", "wav",
        "zip", "tar", "gz", "bz2", "xz", "7z", "rar",
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        "woff", "woff2", "ttf", "otf", "eot",
        "pyc", "pyo", "class", "jar",
    ];

    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| binary_extensions.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn is_hidden(path: &std::path::Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.') && n != "." && n != "..")
        .unwrap_or(false)
}
