use serde_json::json;
use systemprompt_engine::store::{Change, MessageId, Part, PartBody, SessionId};
use systemprompt_harness_views::transcript::children::Children;

fn tool_part(session: &SessionId, finished: bool) -> Part {
    let state = if finished {
        json!({ "status": "completed", "input": { "path": "a.rs" }, "output": "", "title": "a.rs", "started": 1, "ended": 2 })
    } else {
        json!({ "status": "running", "input": { "path": "a.rs" }, "started": 1 })
    };
    let body: PartBody = serde_json::from_value(
        json!({ "type": "tool", "call_id": "call", "tool": "read", "state": state }),
    )
    .unwrap();
    Part::new(
        &crate::IDS,
        session.clone(),
        MessageId::mint(&crate::IDS),
        body,
    )
}

#[test]
fn only_watched_sessions_are_tracked() {
    let mut children = Children::default();
    let child = SessionId::mint(&crate::IDS);
    let other = SessionId::mint(&crate::IDS);
    assert!(children.watch(&child));
    assert!(!children.watch(&child));
    assert!(!children.apply(&Change::PartUpdated(tool_part(&other, false))));
    assert!(children.progress(&other).is_none());
}

#[test]
fn progress_counts_steps_and_names_the_running_one() {
    let mut children = Children::default();
    let child = SessionId::mint(&crate::IDS);
    assert!(children.watch(&child));
    let running = tool_part(&child, false);
    assert!(children.apply(&Change::PartUpdated(running.clone())));
    assert!(!children.apply(&Change::PartUpdated(running)));
    assert!(children.apply(&Change::PartUpdated(tool_part(&child, true))));
    let progress = children.progress(&child).unwrap();
    assert_eq!(progress.steps, 2);
    assert_eq!(progress.current.as_deref(), Some("read a.rs"));
}
