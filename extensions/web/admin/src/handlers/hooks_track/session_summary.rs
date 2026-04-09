use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

use crate::repositories::hooks_track;

#[derive(Debug)]
pub struct SessionSummary {
    pub summary: String,
    pub tags: String,
}

pub async fn generate_session_summary(
    pool: &PgPool,
    user_id: &UserId,
    session_id: &SessionId,
) -> Option<SessionSummary> {
    let rows = hooks_track::fetch_session_events(pool, session_id, user_id)
        .await
        .ok()?;

    if rows.is_empty() {
        return None;
    }

    let mut prompt_count = 0usize;
    let mut tool_count = 0usize;
    let mut error_count = 0usize;
    let mut tools_seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut project: Option<String> = None;

    for row in &rows {
        if row.event_type.contains("UserPromptSubmit") {
            prompt_count += 1;
        } else if row.event_type.contains("PostToolUse") {
            tool_count += 1;
            if let Some(ref t) = row.tool_name {
                tools_seen.insert(t.clone());
            }
        }
        if row.event_type.contains("Failure") {
            error_count += 1;
        }
        if project.is_none() {
            if let Some(ref cwd) = row.cwd {
                if let Some(name) = cwd.rsplit('/').next().filter(|s| !s.is_empty()) {
                    project = Some(name.to_string());
                }
            }
        }
    }

    let summary = build_summary_text(
        prompt_count,
        tool_count,
        error_count,
        &tools_seen,
        project.as_ref(),
    );
    let tags_str = compute_tags(error_count, &tools_seen);

    Some(SessionSummary {
        summary,
        tags: tags_str,
    })
}

fn build_summary_text(
    prompt_count: usize,
    tool_count: usize,
    error_count: usize,
    tools_seen: &std::collections::HashSet<String>,
    project: Option<&String>,
) -> String {
    let mut parts = Vec::new();
    parts.push(format!("{prompt_count} prompts, {tool_count} tool calls"));

    if !tools_seen.is_empty() {
        let mut sorted: Vec<_> = tools_seen.iter().cloned().collect();
        sorted.sort();
        let display: Vec<&str> = sorted.iter().take(5).map(String::as_str).collect();
        let suffix = if sorted.len() > 5 {
            format!(" +{} more", sorted.len() - 5)
        } else {
            String::new()
        };
        parts.push(format!("across {}{}", display.join(", "), suffix));
    }

    if let Some(p) = project {
        parts.push(format!("in {p}"));
    }

    if error_count > 0 {
        parts.push(format!("{error_count} errors"));
    }

    parts.join(". ") + "."
}

fn compute_tags(error_count: usize, tools_seen: &std::collections::HashSet<String>) -> String {
    let mut tags = Vec::new();
    let coding_tools = ["Edit", "Write", "NotebookEdit"];
    let research_tools = ["WebSearch", "WebFetch"];

    if tools_seen
        .iter()
        .any(|t| coding_tools.iter().any(|c| t.contains(c)))
    {
        tags.push("coding");
    }
    if tools_seen
        .iter()
        .any(|t| research_tools.iter().any(|r| t.contains(r)))
    {
        tags.push("research");
    }
    if error_count > 0 {
        tags.push("debugging");
    }
    if tools_seen.iter().any(|t| t.contains("Bash")) {
        tags.push("shell");
    }
    if tools_seen
        .iter()
        .any(|t| t.contains("Read") || t.contains("Grep") || t.contains("Glob"))
    {
        tags.push("exploration");
    }

    tags.join(",")
}
