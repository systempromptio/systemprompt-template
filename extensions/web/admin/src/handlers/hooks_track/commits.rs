//! Detects git commits inside Claude Code Bash tool calls and records them.
//!
//! Best-effort by construction: only commits made through tracked Bash tool
//! calls are visible (other terminals, scripts, and aliases are not); amends
//! and rebases mint new hashes and count as new commits; the stdout formats
//! parsed here (`[branch hash] subject` and the `N files changed, …` stats
//! line) are git's English porcelain — a `--quiet` commit or an unexpected
//! locale records the commit without stats or not at all. Runs on the full
//! in-memory payload, before `sanitize_metadata` truncates the stored copy.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

use crate::repositories::dashboard::commits as commits_repo;
use crate::types::webhook::{HookEvent, HookEventPayload, ToolInputSummary};

use super::helpers::truncate;

pub(super) async fn record_observed_commits(
    pool: &PgPool,
    user_id: &UserId,
    session_id: &SessionId,
    payload: &HookEventPayload,
) {
    let HookEvent::PostToolUse(d) = &payload.event else {
        return;
    };
    if d.name != "Bash" {
        return;
    }
    let Some(command) = ToolInputSummary::of(&d.input).command else {
        return;
    };
    if !is_commit_command(&command) {
        return;
    }
    let stdout = response_stdout(&d.response);
    for parsed in parse_commit_stdout(&stdout) {
        let message = truncate(&parsed.message, 200);
        let result = commits_repo::insert_user_commit(
            pool,
            &commits_repo::NewUserCommit {
                user_id,
                session_id,
                cwd: payload.cwd(),
                branch: parsed.branch.as_deref(),
                commit_hash: &parsed.hash,
                message: &message,
                files_changed: parsed.files_changed,
                insertions: parsed.insertions,
                deletions: parsed.deletions,
            },
        )
        .await;
        if let Err(e) = result {
            tracing::warn!(error = %e, "Failed to record observed commit");
        }
    }
}

pub fn is_commit_command(command: &str) -> bool {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    if tokens.contains(&"--dry-run") {
        return false;
    }
    let mut saw_git = false;
    for t in tokens {
        if t == "git" {
            saw_git = true;
        } else if saw_git && t == "commit" {
            return true;
        }
    }
    false
}

// JSON: protocol boundary — the tool response has no fixed shape; Bash
// responses are usually `{"stdout": …, "stderr": …}` but may be a bare string.
pub fn response_stdout(response: &serde_json::Value) -> String {
    match response {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(map) => map
            .get("stdout")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .unwrap_or_default(),
        _ => String::new(),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParsedCommit {
    pub branch: Option<String>,
    pub hash: String,
    pub message: String,
    pub files_changed: Option<i32>,
    pub insertions: Option<i32>,
    pub deletions: Option<i32>,
}

/// A chained command (`git commit && git commit`) can print several summary
/// lines; each becomes one commit, taking the first stats line that follows it
/// before the next summary line.
pub fn parse_commit_stdout(stdout: &str) -> Vec<ParsedCommit> {
    let lines: Vec<&str> = stdout.lines().collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some(mut commit) = parse_summary_line(line) else {
            continue;
        };
        for follower in lines.iter().skip(i + 1) {
            if parse_summary_line(follower).is_some() {
                break;
            }
            if let Some((files, ins, del)) = parse_stats_line(follower) {
                commit.files_changed = files;
                commit.insertions = ins;
                commit.deletions = del;
                break;
            }
        }
        out.push(commit);
    }
    out
}

// Why: `[main abc1234] subject`, `[main (root-commit) abc1234] subject`,
// `[detached HEAD abc1234] subject` — the hash is the last bracket token.
fn parse_summary_line(line: &str) -> Option<ParsedCommit> {
    let rest = line.strip_prefix('[')?;
    let (bracket, message) = rest
        .split_once("] ")
        .or_else(|| rest.strip_suffix(']').map(|b| (b, "")))?;
    let mut tokens: Vec<&str> = bracket.split_whitespace().collect();
    let hash = tokens.pop()?;
    if hash.len() < 7 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let branch: Vec<&str> = tokens
        .into_iter()
        .filter(|t| *t != "(root-commit)")
        .collect();
    Some(ParsedCommit {
        branch: (!branch.is_empty()).then(|| branch.join(" ")),
        hash: hash.to_owned(),
        message: message.to_owned(),
        files_changed: None,
        insertions: None,
        deletions: None,
    })
}

// Why: `3 files changed, 10 insertions(+), 2 deletions(-)` — every clause is
// optional except the first.
fn parse_stats_line(line: &str) -> Option<(Option<i32>, Option<i32>, Option<i32>)> {
    let trimmed = line.trim();
    if !trimmed.contains("file changed") && !trimmed.contains("files changed") {
        return None;
    }
    let mut files = None;
    let mut insertions = None;
    let mut deletions = None;
    for clause in trimmed.split(',') {
        let mut words = clause.split_whitespace();
        let number: Option<i32> = words.next().and_then(|w| w.parse().ok());
        let Some(unit) = words.next() else { continue };
        if unit.starts_with("file") {
            files = number;
        } else if unit.starts_with("insertion") {
            insertions = number;
        } else if unit.starts_with("deletion") {
            deletions = number;
        }
    }
    files.map(|_| (files, insertions, deletions))
}
