#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

//! `/hooks/track` LOC counting: the line deltas computed from the
//! un-truncated tool input before `sanitize_metadata` destroys it. These pin
//! the tool-shape rules: Write counts content lines as added, Edit counts
//! new/old string lines, MultiEdit sums its edits array, failures and
//! non-edit tools count zero.

use systemprompt_web_admin::test_support::{LocDelta, compute_loc_delta};
use systemprompt_web_admin::types::webhook::HookEventPayload;

fn payload(event: &str, extra: serde_json::Value) -> HookEventPayload {
    let mut base = serde_json::json!({
        "hook_event_name": event,
        "session_id": "sess-loc",
        "cwd": "/tmp/repo",
        "permission_mode": "default",
        "agent_id": null,
    });
    let (serde_json::Value::Object(base_map), serde_json::Value::Object(extra_map)) =
        (&mut base, extra)
    else {
        panic!("both sides must be JSON objects");
    };
    base_map.extend(extra_map);
    let (payload, _warnings) = HookEventPayload::from_value(base);
    payload
}

#[test]
fn write_counts_content_lines_as_added() {
    let p = payload(
        "PostToolUse",
        serde_json::json!({
            "tool_name": "Write",
            "tool_input": { "file_path": "a.rs", "content": "line one\nline two\nline three" },
            "tool_response": {},
        }),
    );
    assert_eq!(
        compute_loc_delta(&p),
        LocDelta {
            added: 3,
            removed: 0
        }
    );
}

#[test]
fn edit_counts_new_and_old_lines() {
    let p = payload(
        "PostToolUse",
        serde_json::json!({
            "tool_name": "Edit",
            "tool_input": {
                "file_path": "a.rs",
                "old_string": "a\nb",
                "new_string": "a\nb\nc\nd",
            },
            "tool_response": {},
        }),
    );
    assert_eq!(
        compute_loc_delta(&p),
        LocDelta {
            added: 4,
            removed: 2
        }
    );
}

#[test]
fn replace_all_still_counts_once() {
    let p = payload(
        "PostToolUse",
        serde_json::json!({
            "tool_name": "Edit",
            "tool_input": {
                "old_string": "x", "new_string": "y", "replace_all": true,
            },
            "tool_response": {},
        }),
    );
    assert_eq!(
        compute_loc_delta(&p),
        LocDelta {
            added: 1,
            removed: 1
        }
    );
}

#[test]
fn multi_edit_sums_the_edits_array() {
    let p = payload(
        "PostToolUse",
        serde_json::json!({
            "tool_name": "MultiEdit",
            "tool_input": {
                "file_path": "a.rs",
                "edits": [
                    { "old_string": "a", "new_string": "a\nb" },
                    { "old_string": "c\nd\ne", "new_string": "c" },
                ],
            },
            "tool_response": {},
        }),
    );
    assert_eq!(
        compute_loc_delta(&p),
        LocDelta {
            added: 3,
            removed: 4
        }
    );
}

#[test]
fn notebook_edit_counts_new_source_added_only() {
    let p = payload(
        "PostToolUse",
        serde_json::json!({
            "tool_name": "NotebookEdit",
            "tool_input": { "new_source": "cell line 1\ncell line 2" },
            "tool_response": {},
        }),
    );
    assert_eq!(
        compute_loc_delta(&p),
        LocDelta {
            added: 2,
            removed: 0
        }
    );
}

#[test]
fn failures_and_non_edit_tools_count_zero() {
    let failure = payload(
        "PostToolUseFailure",
        serde_json::json!({
            "tool_name": "Edit",
            "tool_input": { "old_string": "a", "new_string": "b\nc" },
            "error": "boom",
        }),
    );
    assert_eq!(compute_loc_delta(&failure), LocDelta::default());

    let bash = payload(
        "PostToolUse",
        serde_json::json!({
            "tool_name": "Bash",
            "tool_input": { "command": "echo hi" },
            "tool_response": { "stdout": "hi" },
        }),
    );
    assert_eq!(compute_loc_delta(&bash), LocDelta::default());
}

#[test]
fn empty_strings_count_zero_lines() {
    let p = payload(
        "PostToolUse",
        serde_json::json!({
            "tool_name": "Write",
            "tool_input": { "content": "" },
            "tool_response": {},
        }),
    );
    assert_eq!(compute_loc_delta(&p), LocDelta::default());
}
