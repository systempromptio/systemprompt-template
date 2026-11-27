use harness_desktop::views::transcript::tool_info::{
    Outcome, Phase, TodoLine, ToolKind, exit_code, summarize, todos, truncate,
};
use harness_engine::denied;
use harness_store::{TodoStatus, ToolState};
use serde_json::{Value, json};

fn completed(input: Value, metadata: Value) -> ToolState {
    let Value::Object(metadata) = metadata else {
        panic!("metadata must be an object")
    };
    ToolState::Completed {
        input,
        output: "done".to_owned(),
        title: "title".to_owned(),
        metadata,
        attachments: Vec::new(),
        started: 1_000,
        ended: 1_420,
    }
}

fn failed(error: &str, metadata: &Value) -> ToolState {
    let state = json!({ "status": "error", "input": {}, "error": error, "metadata": metadata, "started": 1, "ended": 2 });
    serde_json::from_value(state).unwrap()
}

fn outcome(state: &ToolState) -> Outcome {
    match summarize("shell", state).phase {
        Phase::Failed(outcome) => outcome,
        other => panic!("expected a failed phase, got {other:?}"),
    }
}

#[test]
fn shell_rows_show_the_first_line_of_the_command() {
    let state = completed(
        json!({ "command": "cargo build\ncargo test" }),
        json!({ "exit_code": 0 }),
    );
    let summary = summarize("shell", &state);
    assert_eq!(summary.kind, ToolKind::Shell);
    assert_eq!(summary.subject.as_deref(), Some("cargo build"));
    assert_eq!(summary.duration_ms, Some(420));
    assert_eq!(summary.phase, Phase::Done);
    assert_eq!(exit_code(&state), Some(0));
}

#[test]
fn patch_rows_list_the_files_from_metadata() {
    let state = completed(
        json!({ "patch": "x" }),
        json!({ "files": ["a.rs", "b.rs"] }),
    );
    assert_eq!(
        summarize("patch", &state).subject.as_deref(),
        Some("a.rs, b.rs")
    );
}

#[test]
fn unknown_tools_are_external() {
    let state = ToolState::Pending { input: json!({}) };
    let summary = summarize("mcp_search", &state);
    assert_eq!(summary.kind, ToolKind::External);
    assert_eq!(summary.phase, Phase::Pending);
    assert!(summary.subject.is_none());
}

#[test]
fn policy_denials_keep_their_reason() {
    let state = failed(
        "blocked",
        &json!({ denied::KEY: denied::POLICY, denied::REASON: "no network" }),
    );
    assert_eq!(
        outcome(&state),
        Outcome::PolicyDenied("no network".to_owned())
    );
}

#[test]
fn rejections_keep_the_feedback() {
    let with = failed(
        "declined",
        &json!({ denied::KEY: denied::REJECTED, denied::FEEDBACK: "use tabs" }),
    );
    assert_eq!(
        outcome(&with),
        Outcome::Rejected(Some("use tabs".to_owned()))
    );
    let without = failed("declined", &json!({ denied::KEY: denied::REJECTED }));
    assert_eq!(outcome(&without), Outcome::Rejected(None));
}

#[test]
fn rule_denials_show_the_reason_or_the_error() {
    let bare = failed(
        "a rule denies shell\nmore",
        &json!({ denied::KEY: denied::RULE }),
    );
    assert_eq!(
        outcome(&bare),
        Outcome::RuleDenied("a rule denies shell".to_owned())
    );
    let reasoned = failed(
        "denied",
        &json!({ denied::KEY: denied::RULE, denied::REASON: "no shell here" }),
    );
    assert_eq!(
        outcome(&reasoned),
        Outcome::RuleDenied("no shell here".to_owned())
    );
}

#[test]
fn aborts_carry_the_marker_and_unmarked_errors_are_failures() {
    let aborted = harness_tools::ToolError::Aborted.to_string();
    assert_eq!(
        outcome(&failed(&aborted, &json!({ denied::KEY: denied::ABORTED }))),
        Outcome::Aborted
    );
    assert_eq!(
        outcome(&failed(&aborted, &json!({}))),
        Outcome::Error(aborted.clone())
    );
    assert_eq!(
        outcome(&failed(" file not found ", &json!({}))),
        Outcome::Error("file not found".to_owned())
    );
    let looks_denied = failed("Denied by policy: Reason: none", &json!({}));
    assert_eq!(
        outcome(&looks_denied),
        Outcome::Error("Denied by policy: Reason: none".to_owned())
    );
}

#[test]
fn todos_are_read_from_metadata() {
    let state = completed(
        json!({}),
        json!({ "todos": [
            { "content": "write tests", "status": "completed" },
            { "content": "ship", "status": "in_progress" },
            { "content": "broken", "status": "unknown" }
        ] }),
    );
    let lines = todos(&state);
    assert_eq!(lines.len(), 2);
    assert_eq!(
        lines[1],
        TodoLine {
            content: "ship".to_owned(),
            status: TodoStatus::InProgress
        }
    );
}

#[test]
fn long_subjects_are_truncated() {
    let long = "x".repeat(300);
    let cut = truncate(&long, 10);
    assert_eq!(cut.chars().count(), 11);
    assert!(cut.ends_with('…'));
    assert_eq!(truncate("short", 10), "short");
}
