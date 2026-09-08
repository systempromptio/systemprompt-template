//! AI-authored line-of-code deltas, measured from the un-truncated tool input.
//!
//! Must run before `sanitize_metadata`: the stored metadata truncates every
//! `tool_input` string to 200 chars, so the counts here are the only durable
//! record of the diff size. Known measurement limits, accepted by design:
//! a `Write` overwriting an existing file counts every line as added (the
//! previous content is not in the hook payload); an `Edit` with
//! `replace_all: true` is counted once (the true multiple is unknowable);
//! `NotebookEdit` removals are unobservable. Failed tool calls count zero —
//! the edit never landed.

use crate::types::webhook::{HookEvent, HookEventPayload};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LocDelta {
    pub added: i64,
    pub removed: i64,
}

pub fn compute_loc_delta(payload: &HookEventPayload) -> LocDelta {
    match &payload.event {
        HookEvent::PostToolUse(d) => delta_for_tool(&d.name, &d.input),
        _ => LocDelta::default(),
    }
}

// JSON: protocol boundary — reads the untyped tool_input carried on the hook
// envelope; shapes vary by tool and client version, so absent keys count zero.
fn delta_for_tool(name: &str, input: &serde_json::Value) -> LocDelta {
    match name {
        "Write" => LocDelta {
            added: line_count(input.get("content")),
            removed: 0,
        },
        "Edit" | "MultiEdit" => edits_delta(input),
        "NotebookEdit" => LocDelta {
            added: line_count(input.get("new_source")),
            removed: 0,
        },
        _ => LocDelta::default(),
    }
}

fn edits_delta(input: &serde_json::Value) -> LocDelta {
    if let Some(edits) = input.get("edits").and_then(serde_json::Value::as_array) {
        return edits.iter().fold(LocDelta::default(), |acc, e| {
            let d = single_edit_delta(e);
            LocDelta {
                added: acc.added.saturating_add(d.added),
                removed: acc.removed.saturating_add(d.removed),
            }
        });
    }
    single_edit_delta(input)
}

// JSON: protocol boundary — one edit object from the untyped tool_input; only
// the two string fields are read, absent keys count zero.
fn single_edit_delta(edit: &serde_json::Value) -> LocDelta {
    LocDelta {
        added: line_count(edit.get("new_string")),
        removed: line_count(edit.get("old_string")),
    }
}

// JSON: protocol boundary — a leaf of the untyped tool_input; non-strings
// count zero rather than erroring on a shape this build has never seen.
fn line_count(value: Option<&serde_json::Value>) -> i64 {
    value
        .and_then(serde_json::Value::as_str)
        .map_or(0, |s| i64::try_from(s.lines().count()).unwrap_or(i64::MAX))
}
