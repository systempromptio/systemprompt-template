//! Hook-event ingest is lenient, and that is a decision rather than an
//! accident.
//!
//! `/hooks/track` and `/hooks/govern` are fed by Claude Code, which ships on
//! its own schedule and adds fields to hook payloads without asking. If a
//! payload that no longer matches its declared shape were rejected, the
//! governance record would silently develop holes exactly when a client
//! upgraded — the failure would show up as missing audit rows, not as an
//! error. So an unparseable or unrecognised event degrades to
//! `HookEvent::Unknown` carrying its name, plus a warning, and is still
//! recorded.
//!
//! These tests pin that contract, because the obvious refactor — an
//! internally-tagged enum behind `Json<HookEventPayload>` — would quietly
//! replace it with a 400 and drop the event.

use serde_json::json;
use systemprompt_web_admin::types::webhook::{HookEvent, HookEventPayload};

fn common() -> serde_json::Value {
    json!({
        "session_id": "sess-1",
        "cwd": "/repo",
    })
}

fn payload(extra: serde_json::Value) -> serde_json::Value {
    let mut base = common();
    let (Some(obj), Some(add)) = (base.as_object_mut(), extra.as_object()) else {
        panic!("both fixtures are JSON objects");
    };
    for (k, v) in add {
        obj.insert(k.clone(), v.clone());
    }
    base
}

#[test]
fn a_well_formed_event_parses_into_its_variant() {
    let (parsed, warnings) = HookEventPayload::from_value(payload(json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": { "command": "ls" },
    })));

    assert!(
        matches!(parsed.event, HookEvent::PreToolUse(_)),
        "a valid PreToolUse payload should reach the PreToolUse variant, got {:?}",
        parsed.event
    );
    assert!(
        warnings.is_empty(),
        "a valid payload should not warn, got {warnings:?}"
    );
}

#[test]
fn an_unrecognised_event_name_is_recorded_under_that_name() {
    let (parsed, warnings) = HookEventPayload::from_value(payload(json!({
        "hook_event_name": "SomethingClaudeCodeAddedLater",
    })));

    assert!(
        matches!(
            &parsed.event,
            HookEvent::Unknown(name) if name == "SomethingClaudeCodeAddedLater"
        ),
        "an unknown event must keep its name so the audit row still identifies it, got {:?}",
        parsed.event
    );
    assert!(
        warnings.iter().any(|w| w.contains("Unknown hook event type")),
        "the unknown event should warn, got {warnings:?}"
    );
}

#[test]
fn a_malformed_known_event_degrades_instead_of_failing() {
    // The variant types default every field, so only a type mismatch — rather
    // than a missing or extra one — can actually fail the parse. That is the
    // point: it takes real corruption to lose an event.
    let (parsed, warnings) = HookEventPayload::from_value(payload(json!({
        "hook_event_name": "PreToolUse",
        "tool_name": 42,
    })));

    assert!(
        matches!(&parsed.event, HookEvent::Unknown(name) if name == "PreToolUse"),
        "a PreToolUse whose body does not fit must still be recorded as PreToolUse, got {:?}",
        parsed.event
    );
    assert!(
        warnings.iter().any(|w| w.contains("PreToolUse parse error")),
        "the parse failure should be reported as a warning, got {warnings:?}"
    );
}

#[test]
fn a_subagent_event_without_an_agent_id_warns_but_still_parses() {
    let (parsed, warnings) = HookEventPayload::from_value(payload(json!({
        "hook_event_name": "SubagentStop",
    })));

    assert!(
        matches!(parsed.event, HookEvent::SubagentStop(_)),
        "the event itself is well-formed, so it should parse, got {:?}",
        parsed.event
    );
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("SubagentStop missing expected common field: agent_id")),
        "agent_id lives on the common fields, so its absence is checked outside the variant \
         parse, got {warnings:?}"
    );
}
