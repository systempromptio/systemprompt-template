use regex::RegexBuilder;
use rmcp::{
    model::{CallToolRequestParam, CallToolResult, Content},
    ErrorData as McpError,
};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use systemprompt::identifiers::McpExecutionId;
use systemprompt::models::artifacts::{ExecutionMetadata, TextArtifact, ToolResponse};
use walkdir::WalkDir;

use crate::constants::{GREP_LINE_TRUNCATE, MAX_GREP_FILES, MAX_GREP_MATCHES};
use crate::error::ToolError;
use crate::SystemToolsServer;

use super::ToolArguments;

struct GrepMatch {
    file: PathBuf,
    line_number: usize,
    line: String,
}

struct GrepConfig<'a> {
    base_path: PathBuf,
    regex: regex::Regex,
    glob_pattern: Option<glob::Pattern>,
    pattern_text: &'a str,
}

pub fn handle(
    request: CallToolRequestParam,
    server: &SystemToolsServer,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let arguments = ToolArguments::new(request.arguments);

    let pattern = arguments.get_required_string("pattern")?;
    let search_path = arguments.get_optional_path("path");
    let file_glob = arguments.get_optional_string("glob");
    let case_insensitive = arguments.get_bool_or("case_insensitive", false);

    let base_path = resolve_base_path(search_path, server)?;
    let regex = build_regex(pattern, case_insensitive)?;
    let glob_pattern = build_glob_pattern(file_glob)?;

    let config = GrepConfig {
        base_path,
        regex,
        glob_pattern,
        pattern_text: pattern,
    };

    let (matches, files_searched) = execute_search(&config)?;
    Ok(build_result(
        &config,
        &matches,
        files_searched,
        mcp_execution_id,
    ))
}

fn resolve_base_path(
    search_path: Option<PathBuf>,
    server: &SystemToolsServer,
) -> Result<PathBuf, McpError> {
    match search_path {
        Some(path) => server
            .validate_path(&path)
            .map_err(|error| McpError::invalid_params(error, None)),
        None => server
            .get_roots()
            .first()
            .cloned()
            .ok_or_else(|| ToolError::NoFileRoots.into()),
    }
}

fn build_regex(pattern: &str, case_insensitive: bool) -> Result<regex::Regex, McpError> {
    RegexBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .build()
        .map_err(|error| {
            ToolError::InvalidRegexPattern {
                details: error.to_string(),
            }
            .into()
        })
}

fn build_glob_pattern(file_glob: Option<&str>) -> Result<Option<glob::Pattern>, McpError> {
    file_glob
        .map(glob::Pattern::new)
        .transpose()
        .map_err(|error| {
            ToolError::InvalidGlobPattern {
                details: error.to_string(),
            }
            .into()
        })
}

fn execute_search(config: &GrepConfig<'_>) -> Result<(Vec<GrepMatch>, usize), McpError> {
    let mut matches: Vec<GrepMatch> = Vec::new();
    let mut files_searched = 0;

    if config.base_path.is_file() {
        if let Some(file_matches) = search_file(&config.base_path, &config.regex, MAX_GREP_MATCHES)
        {
            matches.extend(file_matches);
        }
        return Ok((matches, 1));
    }

    if !config.base_path.is_dir() {
        return Err(ToolError::PathDoesNotExist {
            path: config.base_path.display().to_string(),
            details: "Path is neither a file nor directory".to_string(),
        }
        .into());
    }

    for entry in WalkDir::new(&config.base_path)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if files_searched >= MAX_GREP_FILES || matches.len() >= MAX_GREP_MATCHES {
            break;
        }

        let path = entry.path();
        if !should_search_file(path, config.glob_pattern.as_ref()) {
            continue;
        }

        files_searched += 1;
        if let Some(file_matches) =
            search_file(path, &config.regex, MAX_GREP_MATCHES - matches.len())
        {
            matches.extend(file_matches);
        }
    }

    Ok((matches, files_searched))
}

fn should_search_file(path: &Path, glob_pattern: Option<&glob::Pattern>) -> bool {
    if !path.is_file() || is_likely_binary(path) || is_hidden(path) {
        return false;
    }

    if let Some(glob_pat) = glob_pattern {
        if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
            return glob_pat.matches(file_name);
        }
        return false;
    }

    true
}

fn build_result(
    config: &GrepConfig<'_>,
    matches: &[GrepMatch],
    files_searched: usize,
    mcp_execution_id: &McpExecutionId,
) -> CallToolResult {
    let pattern = config.pattern_text;
    let metadata = ExecutionMetadata::new().tool("grep");
    let artifact_id = uuid::Uuid::new_v4().to_string();

    if matches.is_empty() {
        let artifact_content = format!(
            "No matches found for pattern '{pattern}' in {} ({files_searched} files searched)",
            config.base_path.display(),
        );

        let artifact = TextArtifact::new(&artifact_content).with_title("Grep Search Results");

        let tool_response = ToolResponse::new(
            &artifact_id,
            mcp_execution_id.clone(),
            artifact,
            metadata.clone(),
        );

        return CallToolResult {
            content: vec![Content::text(format!(
                "Found 0 matches for '{pattern}' ({files_searched} files searched)"
            ))],
            is_error: Some(false),
            meta: metadata.to_meta(),
            structured_content: Some(tool_response.to_json()),
        };
    }

    let truncated = matches.len() >= MAX_GREP_MATCHES;
    let output = format_matches(matches);
    let match_count = matches.len();

    let unique_files: std::collections::HashSet<_> =
        matches.iter().map(|match_item| &match_item.file).collect();
    let file_count = unique_files.len();

    let header = if truncated {
        format!(
            "Found {match_count} matches for '{pattern}' (truncated, showing first {MAX_GREP_MATCHES}):\n\n",
        )
    } else {
        format!("Found {match_count} matches for '{pattern}' in {files_searched} files:\n\n",)
    };

    let artifact_content = format!("{header}{output}");

    let artifact = TextArtifact::new(&artifact_content).with_title("Grep Search Results");

    let tool_response = ToolResponse::new(
        &artifact_id,
        mcp_execution_id.clone(),
        artifact,
        metadata.clone(),
    );

    let summary = format!("Found {match_count} matches in {file_count} files for '{pattern}'");

    CallToolResult {
        content: vec![Content::text(summary)],
        is_error: Some(false),
        meta: metadata.to_meta(),
        structured_content: Some(tool_response.to_json()),
    }
}

fn search_file(path: &Path, regex: &regex::Regex, max_matches: usize) -> Option<Vec<GrepMatch>> {
    let content = fs::read_to_string(path).ok()?;
    let mut matches = Vec::new();

    for (line_index, line) in content.lines().enumerate() {
        if regex.is_match(line) {
            matches.push(GrepMatch {
                file: path.to_path_buf(),
                line_number: line_index + 1,
                line: truncate_line(line, GREP_LINE_TRUNCATE),
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

fn format_matches(matches: &[GrepMatch]) -> String {
    let mut output = String::new();
    let mut current_file: Option<&PathBuf> = None;

    for match_result in matches {
        if current_file != Some(&match_result.file) {
            if current_file.is_some() {
                output.push('\n');
            }
            let _ = writeln!(output, "{}:", match_result.file.display());
            current_file = Some(&match_result.file);
        }
        let _ = writeln!(
            output,
            "  {:6}: {}",
            match_result.line_number,
            match_result.line.trim()
        );
    }

    output
}

fn truncate_line(line: &str, max_length: usize) -> String {
    if line.len() <= max_length {
        line.to_string()
    } else {
        format!("{}...", &line[..max_length])
    }
}

fn is_likely_binary(path: &Path) -> bool {
    const BINARY_EXTENSIONS: &[&str] = &[
        "exe", "dll", "so", "dylib", "bin", "obj", "o", "a", "lib", "png", "jpg", "jpeg", "gif",
        "bmp", "ico", "webp", "svg", "mp3", "mp4", "avi", "mov", "mkv", "flac", "wav", "zip",
        "tar", "gz", "bz2", "xz", "7z", "rar", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        "woff", "woff2", "ttf", "otf", "eot", "pyc", "pyo", "class", "jar",
    ];

    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| BINARY_EXTENSIONS.contains(&extension.to_lowercase().as_str()))
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.') && name != "." && name != "..")
}
