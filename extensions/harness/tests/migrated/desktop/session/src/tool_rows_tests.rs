//! Tool rows: the label each tool family takes from its input, how a call
//! moves from running to an outcome, and what a running subagent reports.

use serde_json::json;
use systemprompt_engine::run::denied;
use systemprompt_engine::store::{Change, SessionId};
use systemprompt_harness_views::transcript::children::Children;
use systemprompt_harness_views::transcript::tool_info::{Outcome, Phase, ToolKind, summarize};

use crate::TestResult;
use crate::fixtures::{assistant, state, tool, user};

#[test]
fn read_rows_are_labelled_from_their_path() -> TestResult {
    let read = state(
        json!({ "status": "completed", "input": { "path": "src/main.rs" }, "output": "", "title": "", "started": 1, "ended": 2 }),
    )?;
    let summary = summarize("read", &read);
    assert_eq!(summary.kind, ToolKind::Read);
    assert_eq!(summary.subject.as_deref(), Some("src/main.rs"));
    Ok(())
}

#[test]
fn skill_rows_are_labelled_from_the_skill_name() -> TestResult {
    let skill =
        state(json!({ "status": "running", "input": { "name": "release-notes" }, "started": 1 }))?;
    let summary = summarize("skill", &skill);
    assert_eq!(summary.kind, ToolKind::Skill);
    assert_eq!(summary.subject.as_deref(), Some("release-notes"));
    Ok(())
}

#[test]
fn edit_and_write_rows_name_their_file_while_the_input_streams() -> TestResult {
    for name in ["edit", "write"] {
        let pending = state(json!({ "status": "pending", "input": { "path": "notes/todo.md" } }))?;
        let summary = summarize(name, &pending);
        assert_eq!(summary.phase, Phase::Pending);
        assert_eq!(summary.subject.as_deref(), Some("notes/todo.md"));
    }
    Ok(())
}

#[test]
fn a_write_of_empty_content_still_names_its_file() -> TestResult {
    let done = state(
        json!({ "status": "completed", "input": { "path": "empty.txt", "content": "" }, "output": "", "title": "", "started": 1, "ended": 2 }),
    )?;
    let summary = summarize("write", &done);
    assert_eq!(summary.phase, Phase::Done);
    assert_eq!(summary.subject.as_deref(), Some("empty.txt"));
    Ok(())
}

#[test]
fn every_way_a_call_can_fail_is_told_apart() -> TestResult {
    let failed = |metadata: serde_json::Value| {
        state(
            json!({ "status": "error", "input": {}, "error": "stopped", "metadata": metadata, "started": 1, "ended": 2 }),
        )
    };
    let cases = [
        (
            json!({ denied::KEY: denied::POLICY, denied::REASON: "no network" }),
            Outcome::PolicyDenied("no network".to_owned()),
        ),
        (
            json!({ denied::KEY: denied::RULE, denied::REASON: "rule" }),
            Outcome::RuleDenied("rule".to_owned()),
        ),
        (
            json!({ denied::KEY: denied::REJECTED }),
            Outcome::Rejected(None),
        ),
        (json!({ denied::KEY: denied::ABORTED }), Outcome::Aborted),
        (json!({}), Outcome::Error("stopped".to_owned())),
    ];
    for (metadata, expected) in cases {
        assert_eq!(
            summarize("shell", &failed(metadata)?).phase,
            Phase::Failed(expected)
        );
    }
    Ok(())
}

#[test]
fn shell_and_question_calls_go_from_running_to_failed() -> TestResult {
    for name in ["shell", "question"] {
        let running =
            state(json!({ "status": "running", "input": { "command": "make" }, "started": 1 }))?;
        let failed = state(
            json!({ "status": "error", "input": { "command": "make" }, "error": "exit 2", "started": 1, "ended": 5 }),
        )?;
        assert_eq!(summarize(name, &running).phase, Phase::Running);
        let ended = summarize(name, &failed);
        assert_eq!(
            ended.phase,
            Phase::Failed(Outcome::Error("exit 2".to_owned()))
        );
        assert_eq!(ended.duration_ms, Some(4));
    }
    Ok(())
}

#[test]
fn a_running_subagent_reports_what_it_is_doing() -> TestResult {
    let child = SessionId::mint(&crate::IDS);
    let prompt = user(&child);
    let reply = assistant(&child, &prompt.id);
    let mut children = Children::default();
    assert!(children.watch(&child));
    let first = tool(
        &child,
        &reply.id,
        "read",
        state(
            json!({ "status": "completed", "input": { "path": "a.rs" }, "output": "", "title": "", "started": 1, "ended": 2 }),
        )?,
    )?;
    let second = tool(
        &child,
        &reply.id,
        "shell",
        state(json!({ "status": "running", "input": { "command": "cargo test" }, "started": 3 }))?,
    )?;
    assert!(children.apply(&Change::PartUpdated(first)));
    assert!(children.apply(&Change::PartUpdated(second)));
    let progress = children
        .progress(&child)
        .ok_or("progress of a watched child")?;
    assert_eq!(progress.steps, 2);
    assert_eq!(progress.current.as_deref(), Some("shell cargo test"));
    Ok(())
}
