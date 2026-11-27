//! What a tool call shows in its row: the kind of tool, its salient argument,
//! where it stands and how it ended, read from the stored tool state without
//! any GPUI.

use systemprompt_engine::run::denied;
use systemprompt_engine::store::{JsonMap, SessionId, TodoStatus, ToolState};
use systemprompt_harness_shared::IconName;
// JSON: a tool call's input and metadata are the tool's own JSON, stored by
// the engine as the model and the tool wrote them.
use serde_json::Value;

const SUBJECT_LIMIT: usize = 160;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    Shell,
    Read,
    Write,
    Edit,
    Patch,
    Glob,
    Grep,
    Fetch,
    Todo,
    Task,
    Skill,
    Question,
    External,
}

impl ToolKind {
    pub(crate) fn of(tool: &str) -> Self {
        match tool {
            "shell" => Self::Shell,
            "read" => Self::Read,
            "write" => Self::Write,
            "edit" => Self::Edit,
            "patch" => Self::Patch,
            "glob" => Self::Glob,
            "grep" => Self::Grep,
            "fetch" => Self::Fetch,
            "todo" => Self::Todo,
            "task" => Self::Task,
            "skill" => Self::Skill,
            "question" => Self::Question,
            _ => Self::External,
        }
    }

    pub(crate) const fn icon(self) -> IconName {
        match self {
            Self::Shell => IconName::Terminal,
            Self::Read | Self::Write | Self::Edit | Self::Patch => IconName::File,
            Self::Glob | Self::Grep => IconName::Search,
            Self::Fetch => IconName::Globe,
            Self::Todo => IconName::ListTodo,
            Self::Task => IconName::Branch,
            Self::Skill => IconName::Sparkle,
            Self::Question => IconName::Info,
            Self::External => IconName::Plug,
        }
    }

    pub(crate) const fn writes_files(self) -> bool {
        matches!(self, Self::Write | Self::Edit | Self::Patch)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    PolicyDenied(String),
    RuleDenied(String),
    Rejected(Option<String>),
    Aborted,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Pending,
    Running,
    Done,
    Failed(Outcome),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSummary {
    pub kind: ToolKind,
    pub(crate) name: String,
    pub subject: Option<String>,
    pub(crate) title: Option<String>,
    pub phase: Phase,
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoLine {
    pub content: String,
    pub status: TodoStatus,
}

pub fn summarize(tool: &str, state: &ToolState) -> ToolSummary {
    let kind = ToolKind::of(tool);
    let (phase, title, duration_ms) = match state {
        ToolState::Pending { .. } => (Phase::Pending, None, None),
        ToolState::Running { title, .. } => (Phase::Running, title.clone(), None),
        ToolState::Completed {
            title,
            started,
            ended,
            ..
        } => (
            Phase::Done,
            Some(title.clone()),
            Some(ended.saturating_sub(*started)),
        ),
        ToolState::Error {
            error,
            metadata,
            started,
            ended,
            ..
        } => (
            Phase::Failed(outcome(error, metadata)),
            None,
            Some(ended.saturating_sub(*started)),
        ),
    };
    ToolSummary {
        kind,
        name: tool.to_owned(),
        subject: subject(kind, state.input(), metadata(state)),
        title: title.filter(|title| !title.trim().is_empty()),
        phase,
        duration_ms,
    }
}

pub(crate) const fn metadata(state: &ToolState) -> Option<&JsonMap> {
    match state {
        ToolState::Running { metadata, .. }
        | ToolState::Completed { metadata, .. }
        | ToolState::Error { metadata, .. } => Some(metadata),
        ToolState::Pending { .. } => None,
    }
}

// JSON: the tool input as the model sent it and the metadata the tool reported.
// JSON: a tool call's input and metadata are the tool's own JSON, stored by
// the engine as the model and the tool wrote them.
fn subject(kind: ToolKind, input: &Value, metadata: Option<&JsonMap>) -> Option<String> {
    let text = |key: &str| input.get(key).and_then(Value::as_str).map(str::to_owned);
    let found = match kind {
        ToolKind::Shell => {
            text("command").map(|command| command.lines().next().unwrap_or_default().to_owned())
        },
        ToolKind::Read | ToolKind::Write | ToolKind::Edit => text("path"),
        ToolKind::Patch => metadata
            .and_then(|map| map.get("files"))
            .and_then(Value::as_array)
            .map(|files| {
                files
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            }),
        ToolKind::Glob | ToolKind::Grep => text("pattern"),
        ToolKind::Fetch => text("url"),
        ToolKind::Task => text("description"),
        ToolKind::Skill => text("name"),
        ToolKind::Todo | ToolKind::Question | ToolKind::External => None,
    };
    found
        .map(|value| truncate(value.trim(), SUBJECT_LIMIT))
        .filter(|value| !value.is_empty())
}

pub fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_owned();
    }
    let mut cut: String = text.chars().take(limit).collect();
    cut.push('…');
    cut
}

// JSON: the tool part's metadata, where the engine marks a refused call.
// JSON: a tool call's input and metadata are the tool's own JSON, stored by
// the engine as the model and the tool wrote them.
fn outcome(error: &str, metadata: &JsonMap) -> Outcome {
    let detail = |key: &str| {
        metadata
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    };
    let error = error.trim();
    match metadata.get(denied::KEY).and_then(Value::as_str) {
        Some(denied::POLICY) => Outcome::PolicyDenied(detail(denied::REASON).unwrap_or_default()),
        Some(denied::RULE) => {
            Outcome::RuleDenied(detail(denied::REASON).unwrap_or_else(|| first_line(error)))
        },
        Some(denied::REJECTED) => Outcome::Rejected(detail(denied::FEEDBACK)),
        Some(denied::ABORTED) => Outcome::Aborted,
        _ => Outcome::Error(error.to_owned()),
    }
}

fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or_default().trim().to_owned()
}

// JSON: a tool call's input and metadata are the tool's own JSON, stored by
// the engine as the model and the tool wrote them.
pub(crate) fn metadata_text<'a>(state: &'a ToolState, key: &str) -> Option<&'a str> {
    metadata(state)
        .and_then(|map| map.get(key))
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
}

pub(crate) fn output(state: &ToolState) -> Option<&str> {
    match state {
        ToolState::Completed { output, .. } => Some(output.as_str()),
        ToolState::Running { .. } => metadata_text(state, "output"),
        ToolState::Pending { .. } | ToolState::Error { .. } => None,
    }
}

pub fn exit_code(state: &ToolState) -> Option<i64> {
    metadata(state)
        .and_then(|map| map.get("exit_code"))
        .and_then(Value::as_i64)
}

pub(crate) fn child_session(state: &ToolState) -> Option<SessionId> {
    metadata_text(state, "session_id").and_then(|id| SessionId::parse(id).ok())
}

// JSON: a tool call's input and metadata are the tool's own JSON, stored by
// the engine as the model and the tool wrote them.
pub fn todos(state: &ToolState) -> Vec<TodoLine> {
    let listed = metadata(state)
        .and_then(|map| map.get("todos"))
        .or_else(|| state.input().get("todos"))
        .and_then(Value::as_array);
    listed
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let content = item.get("content").and_then(Value::as_str)?.to_owned();
                    let status = item
                        .get("status")
                        .and_then(Value::as_str)
                        .and_then(TodoStatus::parse)?;
                    Some(TodoLine { content, status })
                })
                .collect()
        })
        .unwrap_or_default()
}
