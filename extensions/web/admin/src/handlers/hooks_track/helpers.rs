//! Payload trimming shared by the hook ingestion path.

use crate::types::webhook::{HookEvent, HookEventPayload, ToolInputSummary};

pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_owned()
    } else {
        let end = s.char_indices().nth(max).map_or(s.len(), |(i, _)| i);
        format!("{}…", &s[..end])
    }
}

pub(super) fn generate_prompt_preview(payload: &HookEventPayload) -> Option<String> {
    match &payload.event {
        HookEvent::UserPromptSubmit(d) if !d.prompt.is_empty() => Some(truncate(&d.prompt, 200)),
        HookEvent::Stop(d) => d
            .last_assistant_message
            .as_ref()
            .filter(|m| !m.is_empty())
            .map(|m| truncate(m, 500)),
        HookEvent::SubagentStop(d) => d
            .last_assistant_message
            .as_ref()
            .filter(|m| !m.is_empty())
            .map(|m| truncate(m, 500)),
        _ => None,
    }
}

pub(super) fn compute_content_bytes(payload: &HookEventPayload) -> crate::types::ContentBytes {
    let mut input_bytes: i64 = 0;
    let mut output_bytes: i64 = 0;

    match &payload.event {
        HookEvent::UserPromptSubmit(d) => {
            input_bytes += i64::try_from(d.prompt.len()).unwrap_or(0);
        },
        HookEvent::PreToolUse(d) => {
            input_bytes += i64::try_from(d.input.to_string().len()).unwrap_or(0);
        },
        HookEvent::PostToolUse(d) => {
            input_bytes += i64::try_from(d.input.to_string().len()).unwrap_or(0);
            output_bytes += i64::try_from(d.response.to_string().len()).unwrap_or(0);
        },
        HookEvent::PostToolUseFailure(d) => {
            input_bytes += i64::try_from(d.tool_input.to_string().len()).unwrap_or(0);
            output_bytes += i64::try_from(d.error.len()).unwrap_or(0);
        },
        HookEvent::Stop(d) => {
            if let Some(msg) = &d.last_assistant_message {
                output_bytes += i64::try_from(msg.len()).unwrap_or(0);
            }
        },
        HookEvent::SubagentStop(d) => {
            if let Some(msg) = &d.last_assistant_message {
                output_bytes += i64::try_from(msg.len()).unwrap_or(0);
            }
        },
        _ => {},
    }

    crate::types::ContentBytes {
        input: input_bytes,
        output: output_bytes,
    }
}

pub(super) fn derive_title(message: &str) -> String {
    let first_line = message.lines().next().unwrap_or("");
    let clean = first_line.trim_start_matches('#').trim();
    truncate(clean, 100)
}

pub(super) fn extract_file_path(payload: &HookEventPayload) -> Option<String> {
    match &payload.event {
        HookEvent::PostToolUse(d) => ToolInputSummary::of(&d.input).file_path,
        HookEvent::PostToolUseFailure(d) => ToolInputSummary::of(&d.tool_input).file_path,
        _ => None,
    }
}

// JSON: protocol boundary — trims the raw third-party hook envelope in place;
// the payload has no fixed shape to deserialize into, and the trimming must
// survive keys this build has never seen.
pub(super) fn sanitize_metadata(raw: &serde_json::Value) -> serde_json::Value {
    let mut obj = match raw.as_object() {
        Some(map) => map.clone(),
        None => return raw.clone(),
    };

    obj.retain(|_, v| !v.is_null());
    obj.remove("tool_response");
    obj.remove("prompt");
    obj.remove("last_assistant_message");

    if let Some(tool_input) = obj.remove("tool_input") {
        obj.insert(
            "tool_input".to_owned(),
            truncate_json_value(&tool_input, 200),
        );
    }

    serde_json::Value::Object(obj)
}

// JSON: protocol boundary — walks an arbitrary third-party tool payload to cap
// its stored size without knowing its shape.
fn truncate_json_value(value: &serde_json::Value, max_chars: usize) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut result = serde_json::Map::new();
            for (k, v) in map {
                match v {
                    serde_json::Value::Null => {},
                    serde_json::Value::String(s) if s.len() > max_chars => {
                        let end = s.char_indices().nth(max_chars).map_or(s.len(), |(i, _)| i);
                        result.insert(
                            k.clone(),
                            serde_json::Value::String(format!("{}...[truncated]", &s[..end])),
                        );
                    },
                    _ => {
                        let serialized = v.to_string();
                        if serialized.len() > max_chars {
                            let end = serialized
                                .char_indices()
                                .nth(max_chars)
                                .map_or(serialized.len(), |(i, _)| i);
                            result.insert(
                                k.clone(),
                                serde_json::Value::String(format!(
                                    "{}...[truncated]",
                                    &serialized[..end]
                                )),
                            );
                        } else {
                            result.insert(k.clone(), v.clone());
                        }
                    },
                }
            }
            // JSON: protocol boundary — see the function comment above.
            serde_json::Value::Object(result)
        },
        // JSON: protocol boundary — see the function comment above.
        serde_json::Value::String(s) if s.len() > max_chars => {
            let end = s.char_indices().nth(max_chars).map_or(s.len(), |(i, _)| i);
            serde_json::Value::String(format!("{}...[truncated]", &s[..end]))
        },
        _ => value.clone(),
    }
}
