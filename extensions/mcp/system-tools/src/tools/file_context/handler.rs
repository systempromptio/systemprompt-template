use anyhow::Result;
use rmcp::{
    model::{CallToolRequestParam, CallToolResult, Content},
    ErrorData as McpError,
};
use serde_json::json;
use std::path::PathBuf;
use systemprompt::identifiers::{ArtifactId, McpExecutionId};
use systemprompt::models::artifacts::{ExecutionMetadata, TextArtifact, ToolResponse};
use systemprompt::models::execution::context::RequestContext;
use tracing::{debug, info, warn};

use super::ai_caller::generate_structured;
use super::models::{AccumulatedContext, FileContent, NextAction, ReasoningDecision, SearchResult};
use crate::tools::ToolArguments;
use crate::SystemToolsServer;

const SKILL_ID: &str = "file_context_reasoning";
const SKILL_NAME: &str = "File Context Reasoning";
const DEFAULT_MAX_ITERATIONS: u32 = 5;
const MAX_ITERATIONS_LIMIT: u32 = 10;
const MAX_FILE_CONTENT_CHARS: usize = 50000;
const MAX_CONTEXT_TOKENS: usize = 100000;

pub async fn handle(
    request: CallToolRequestParam,
    server: &SystemToolsServer,
    mcp_execution_id: &McpExecutionId,
    ctx: RequestContext,
) -> Result<CallToolResult, McpError> {
    let arguments = ToolArguments::new(request.arguments);

    let query = arguments
        .get_required_string("query")
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let starting_path = arguments.get_optional_path("path");
    let max_iterations = arguments
        .get_i64_or("max_iterations", i64::from(DEFAULT_MAX_ITERATIONS))
        .min(i64::from(MAX_ITERATIONS_LIMIT)) as u32;

    let root_path = match starting_path {
        Some(p) => server
            .validate_path(&p)
            .map_err(|e| McpError::invalid_params(e, None))?,
        None => server
            .get_roots()
            .first()
            .cloned()
            .ok_or_else(|| McpError::invalid_params("No file roots configured", None))?,
    };

    info!(query = query, "Starting context gathering");

    let skill_content = server
        .skill_service
        .load_skill(SKILL_ID, &ctx)
        .await
        .map_err(|e| {
            McpError::internal_error(format!("Failed to load skill '{}': {}", SKILL_ID, e), None)
        })?;

    if skill_content.is_empty() {
        return Err(McpError::internal_error(
            format!("Skill '{}' not found or empty", SKILL_ID),
            None,
        ));
    }

    let mut context = AccumulatedContext::new();

    let initial_tree = build_directory_tree(&root_path, 3)?;
    context.directory_tree = initial_tree.clone();
    context
        .actions_taken
        .push(format!("Listed directory: {}", root_path.display()));

    let schema = reasoning_decision_schema();

    let mut iteration = 1u32;
    let mut final_result = String::new();

    while iteration <= max_iterations {
        debug!(iteration, max_iterations, "Reasoning iteration");

        if context.estimated_tokens() > MAX_CONTEXT_TOKENS {
            warn!("Context too large, forcing completion");
            final_result = format!(
                "Context limit reached after {} iterations. Current understanding:\n\n{}",
                iteration,
                context.format_for_ai()
            );
            break;
        }

        let user_prompt = build_user_prompt(query, &context, iteration, max_iterations);

        let decision: ReasoningDecision = generate_structured(
            &server.ai_service,
            &skill_content,
            &user_prompt,
            schema.clone(),
            ctx.clone(),
        )
        .await
        .map_err(|e| McpError::internal_error(format!("AI reasoning failed: {}", e), None))?;

        debug!(analysis = %decision.analysis, "AI analysis");

        if decision.is_complete {
            final_result = decision.final_result.unwrap_or(decision.analysis);
            break;
        }

        for action in decision.next_actions {
            execute_action(&action, server, &root_path, &mut context).await?;
        }

        iteration += 1;
    }

    if final_result.is_empty() {
        final_result = format!(
            "Reached maximum iterations ({}). Current understanding:\n\n{}",
            max_iterations,
            context.format_for_ai()
        );
    }

    let metadata = ExecutionMetadata::with_request(&ctx)
        .tool("file_context")
        .skill(SKILL_ID, SKILL_NAME);

    let artifact_id = ArtifactId::new(uuid::Uuid::new_v4().to_string());

    let artifact_content = format!(
        "# File Context Analysis\n\n## Query\n{}\n\n## Result\n{}\n\n## Context Gathered\n{}",
        query,
        final_result,
        context.format_for_ai()
    );

    let artifact =
        TextArtifact::new(&artifact_content).with_title(format!("Context: {}", truncate(query, 50)));

    let tool_response = ToolResponse::new(
        artifact_id,
        mcp_execution_id.clone(),
        artifact,
        metadata.clone(),
    );

    info!(
        iterations = iteration.min(max_iterations),
        skill = SKILL_ID,
        "Completed context gathering"
    );

    Ok(CallToolResult {
        content: vec![Content::text(final_result)],
        is_error: Some(false),
        meta: metadata.to_meta(),
        structured_content: Some(tool_response.to_json()),
    })
}

fn build_user_prompt(
    query: &str,
    context: &AccumulatedContext,
    iteration: u32,
    max_iterations: u32,
) -> String {
    format!(
        r#"## Query
{query}

## Iteration
{iteration} of {max_iterations}

## Accumulated Context
{context}

Based on the above context, analyze what you know and decide:
1. If you have enough information to answer the query comprehensively, set is_complete=true and provide final_result
2. If you need more context, specify next_actions to gather it

Respond with valid JSON matching the schema."#,
        context = context.format_for_ai()
    )
}

fn reasoning_decision_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "analysis": {
                "type": "string",
                "description": "Current understanding and reasoning"
            },
            "is_complete": {
                "type": "boolean",
                "description": "Whether enough context has been gathered"
            },
            "next_actions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "action_type": {
                            "type": "string",
                            "enum": ["read_files", "grep", "list_directory", "glob_search"]
                        },
                        "paths": {"type": "array", "items": {"type": "string"}},
                        "pattern": {"type": "string"},
                        "path": {"type": "string"},
                        "glob": {"type": "string"},
                        "depth": {"type": "integer"}
                    },
                    "required": ["action_type"]
                }
            },
            "final_result": {
                "type": "string",
                "description": "Final synthesized result when is_complete is true"
            }
        },
        "required": ["analysis", "is_complete", "next_actions"]
    })
}

async fn execute_action(
    action: &NextAction,
    server: &SystemToolsServer,
    root_path: &PathBuf,
    context: &mut AccumulatedContext,
) -> Result<(), McpError> {
    match action {
        NextAction::ReadFiles { paths } => {
            for path_str in paths {
                let path = if std::path::Path::new(path_str).is_absolute() {
                    PathBuf::from(path_str)
                } else {
                    root_path.join(path_str)
                };

                match read_file_content(&path, server) {
                    Ok((content, truncated)) => {
                        context.file_contents.push(FileContent {
                            path: path_str.clone(),
                            content,
                            truncated,
                        });
                        context
                            .actions_taken
                            .push(format!("Read file: {}", path_str));
                    }
                    Err(e) => {
                        warn!(path = %path_str, error = %e, "Failed to read file");
                        context
                            .actions_taken
                            .push(format!("Failed to read {}: {}", path_str, e));
                    }
                }
            }
        }

        NextAction::Grep { pattern, path, glob } => {
            let search_path = path
                .as_ref()
                .map(|p| {
                    if std::path::Path::new(p).is_absolute() {
                        PathBuf::from(p)
                    } else {
                        root_path.join(p)
                    }
                })
                .unwrap_or_else(|| root_path.clone());

            match execute_grep(pattern, &search_path, glob.as_deref(), server) {
                Ok(matches) => {
                    context.search_results.push(SearchResult {
                        query: pattern.clone(),
                        matches,
                    });
                    context.actions_taken.push(format!("Searched for: {}", pattern));
                }
                Err(e) => {
                    warn!(pattern = %pattern, error = %e, "Grep failed");
                    context
                        .actions_taken
                        .push(format!("Grep failed for {}: {}", pattern, e));
                }
            }
        }

        NextAction::ListDirectory { path, depth } => {
            let dir_path = if std::path::Path::new(path).is_absolute() {
                PathBuf::from(path)
            } else {
                root_path.join(path)
            };

            match server.validate_path(&dir_path) {
                Ok(validated) => {
                    let tree = build_directory_tree(&validated, depth.unwrap_or(2) as usize)?;
                    context.directory_tree.push_str(&format!("\n\n### {}\n{}", path, tree));
                    context.actions_taken.push(format!("Listed directory: {}", path));
                }
                Err(e) => {
                    warn!(path = %path, error = %e, "Cannot list directory");
                    context
                        .actions_taken
                        .push(format!("Cannot list {}: {}", path, e));
                }
            }
        }

        NextAction::GlobSearch { pattern, path } => {
            let search_path = path
                .as_ref()
                .map(|p| {
                    if std::path::Path::new(p).is_absolute() {
                        PathBuf::from(p)
                    } else {
                        root_path.join(p)
                    }
                })
                .unwrap_or_else(|| root_path.clone());

            match execute_glob(pattern, &search_path, server) {
                Ok(matches) => {
                    context.search_results.push(SearchResult {
                        query: format!("glob:{}", pattern),
                        matches,
                    });
                    context
                        .actions_taken
                        .push(format!("Glob search: {}", pattern));
                }
                Err(e) => {
                    warn!(pattern = %pattern, error = %e, "Glob failed");
                    context
                        .actions_taken
                        .push(format!("Glob failed for {}: {}", pattern, e));
                }
            }
        }
    }

    Ok(())
}

fn read_file_content(path: &PathBuf, server: &SystemToolsServer) -> Result<(String, bool)> {
    let validated = server
        .validate_path(path)
        .map_err(|e| anyhow::anyhow!(e))?;

    let content = std::fs::read_to_string(&validated)?;
    let truncated = content.len() > MAX_FILE_CONTENT_CHARS;

    let final_content = if truncated {
        format!(
            "{}...\n\n[Truncated - {} total characters]",
            &content[..MAX_FILE_CONTENT_CHARS],
            content.len()
        )
    } else {
        content
    };

    Ok((final_content, truncated))
}

fn build_directory_tree(path: &PathBuf, max_depth: usize) -> Result<String, McpError> {
    let mut output = String::new();
    build_tree_recursive(path, &mut output, "", 0, max_depth)?;
    Ok(output)
}

fn build_tree_recursive(
    path: &PathBuf,
    output: &mut String,
    prefix: &str,
    depth: usize,
    max_depth: usize,
) -> Result<(), McpError> {
    use std::fmt::Write;

    if depth > max_depth {
        return Ok(());
    }

    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    let mut items: Vec<_> = entries.filter_map(Result::ok).collect();
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
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str.starts_with('.') {
            continue;
        }

        let connector = if is_last { "└── " } else { "├── " };
        let is_dir = entry.path().is_dir();

        if is_dir {
            let _ = writeln!(output, "{}{}{}/", prefix, connector, name_str);
            if depth < max_depth {
                let new_prefix = if is_last {
                    format!("{}    ", prefix)
                } else {
                    format!("{}│   ", prefix)
                };
                build_tree_recursive(&entry.path(), output, &new_prefix, depth + 1, max_depth)?;
            }
        } else {
            let _ = writeln!(output, "{}{}{}", prefix, connector, name_str);
        }
    }

    Ok(())
}

fn execute_grep(
    pattern: &str,
    path: &PathBuf,
    glob_filter: Option<&str>,
    server: &SystemToolsServer,
) -> Result<Vec<String>> {
    let validated = server
        .validate_path(path)
        .map_err(|e| anyhow::anyhow!(e))?;

    let regex = regex::Regex::new(pattern)?;
    let mut matches = Vec::new();

    let walker = walkdir::WalkDir::new(&validated)
        .max_depth(10)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file());

    for entry in walker {
        let file_path = entry.path();

        if let Some(glob_pattern) = glob_filter {
            let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !glob::Pattern::new(glob_pattern)
                .map(|p| p.matches(file_name))
                .unwrap_or(false)
            {
                continue;
            }
        }

        if let Ok(content) = std::fs::read_to_string(file_path) {
            for (line_num, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    matches.push(format!(
                        "{}:{}: {}",
                        file_path.display(),
                        line_num + 1,
                        truncate(line, 200)
                    ));

                    if matches.len() >= 100 {
                        return Ok(matches);
                    }
                }
            }
        }
    }

    Ok(matches)
}

fn execute_glob(pattern: &str, path: &PathBuf, server: &SystemToolsServer) -> Result<Vec<String>> {
    let validated = server
        .validate_path(path)
        .map_err(|e| anyhow::anyhow!(e))?;

    let full_pattern = validated.join(pattern);
    let pattern_str = full_pattern.to_string_lossy();

    let mut matches = Vec::new();
    for entry in glob::glob(&pattern_str)? {
        if let Ok(path) = entry {
            matches.push(path.display().to_string());
            if matches.len() >= 100 {
                break;
            }
        }
    }

    Ok(matches)
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
