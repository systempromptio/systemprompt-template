//! The view rows a thread walk emits: one tool step, and a finished turn with
//! its tool-name chips.

use chrono::{DateTime, Utc};

use super::conversation::{
    PROMPT_SHORT_CHARS, StepView, ToolChipView, TurnView, is_long, local_time, single_line,
};
use super::markers::{ToolUseMarker, pretty_json_text};
use super::pretty_json;
use crate::repositories::analytics::context_tool_calls::ContextToolCallRow;

pub(super) fn turn_view(
    number: usize,
    prompt: String,
    ts: DateTime<Utc>,
    steps: Vec<StepView>,
) -> TurnView {
    let mut chips: Vec<ToolChipView> = Vec::new();
    for name in steps.iter().filter_map(|s| s.tool_name.as_deref()) {
        if let Some(chip) = chips.iter_mut().find(|c| c.name == name) {
            chip.count += 1;
            continue;
        }
        chips.push(ToolChipView {
            name: name.to_owned(),
            count: 1,
        });
    }
    TurnView {
        number,
        anchor: format!("turn-{number}"),
        prompt_short: single_line(&prompt, PROMPT_SHORT_CHARS),
        prompt_is_long: is_long(&prompt),
        prompt,
        ts_local: local_time(ts),
        ts_full: ts.to_rfc3339(),
        steps,
        tool_names: chips,
    }
}

pub(super) fn tool_step(
    marker: &ToolUseMarker,
    row: Option<&ContextToolCallRow>,
    request_id_short: Option<String>,
    request_url: Option<String>,
) -> StepView {
    StepView {
        is_assistant: false,
        is_tool: true,
        text: None,
        text_is_long: false,
        tool_name: Some(row.map_or_else(|| marker.name.clone(), |r| r.tool_name.clone())),
        tool_input_pretty: Some(row.map_or_else(
            || pretty_json_text(&marker.input_json),
            |r| pretty_json(&r.tool_input),
        )),
        tool_result_pretty: row
            .and_then(|r| r.tool_result_payload.as_ref())
            .map(pretty_json),
        artifact_url: row
            .and_then(|r| r.artifact_id.as_ref())
            .map(|id| format!("/admin/artifacts/{id}")),
        artifact_structured: row.is_some_and(|r| r.artifact_structured),
        meta: None,
        request_id_short,
        request_url,
    }
}
