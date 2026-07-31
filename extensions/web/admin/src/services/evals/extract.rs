//! Pulling readable text out of stored gateway payloads.
//!
//! `ai_request_payloads.request_body` is whatever the client sent to
//! `/v1/messages` and `response_body` is what came back, so both are
//! Anthropic-shaped: `content` is either a bare string or a list of typed
//! blocks. Everything here degrades to the stored excerpt rather than failing,
//! because a payload we cannot parse is still worth flagging.

use serde_json::Value;

#[must_use]
pub(crate) fn final_user_prompt(request_body: Option<&Value>) -> Option<String> {
    let messages = request_body?.get("messages")?.as_array()?;
    messages
        .iter()
        .rev()
        .find(|m| m.get("role").and_then(Value::as_str) == Some("user"))
        .and_then(|m| m.get("content"))
        .map(flatten_content)
        .filter(|s| !s.trim().is_empty())
}

#[must_use]
pub(crate) fn assistant_answer(response_body: Option<&Value>) -> Option<String> {
    let body = response_body?;
    let content = body.get("content").or_else(|| body.get("completion"))?;
    let text = flatten_content(content);
    (!text.trim().is_empty()).then_some(text)
}

// Why: a non-zero count with no text is a normal agentic turn, not an empty
// answer.
#[must_use]
pub(crate) fn tool_use_count(response_body: Option<&Value>) -> usize {
    response_body
        .and_then(|b| b.get("content"))
        .and_then(Value::as_array)
        .map_or(0, |blocks| {
            blocks
                .iter()
                .filter(|b| b.get("type").and_then(Value::as_str) == Some("tool_use"))
                .count()
        })
}

#[must_use]
pub(crate) fn stop_reason(response_body: Option<&Value>) -> Option<String> {
    response_body?
        .get("stop_reason")?
        .as_str()
        .map(str::to_owned)
}

fn flatten_content(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(blocks) => blocks
            .iter()
            .filter_map(flatten_block)
            .collect::<Vec<_>>()
            .join("\n"),
        other => other.to_string(),
    }
}

fn flatten_block(block: &Value) -> Option<String> {
    match block.get("type").and_then(Value::as_str) {
        Some("text") => block
            .get("text")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .filter(|s| !s.is_empty()),
        Some("tool_use") => {
            let name = block
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("(unnamed)");
            let input = block
                .get("input")
                .map(ToString::to_string)
                .unwrap_or_default();
            Some(format!("[tool_use {name}] {input}"))
        },
        Some("tool_result") => {
            let inner = block
                .get("content")
                .map(flatten_content)
                .unwrap_or_default();
            Some(format!("[tool_result] {inner}"))
        },
        Some("thinking") => None,
        _ => block
            .get("text")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .filter(|s| !s.is_empty()),
    }
}

// Why: long transcripts cost judge tokens without improving the verdict, and
// the tail is where the answer actually is, so keep head and tail rather than a
// prefix.
#[must_use]
pub(crate) fn truncate_for_judge(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let head: String = text.chars().take(max_chars / 2).collect();
    let tail: String = text
        .chars()
        .skip(text.chars().count() - max_chars / 2)
        .collect();
    format!("{head}\n\n[… {} characters elided …]\n\n{tail}", {
        text.chars().count() - max_chars
    })
}

#[must_use]
pub(crate) fn excerpt(text: &str, max_chars: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= max_chars {
        collapsed
    } else {
        let head: String = collapsed.chars().take(max_chars).collect();
        format!("{head}…")
    }
}

// Why: a streamed turn stores no `response_body` at all — only the raw SSE
// wire text in `response_excerpt` — so without reassembly the judge would be
// shown protocol frames and would (correctly, but uselessly) fail every
// streamed answer for "wrong format".
#[must_use]
pub(crate) fn assistant_answer_from_sse(sse: &str) -> Option<StreamedAnswer> {
    let mut text = String::new();
    let mut saw_stream = false;
    let mut complete = false;

    for line in sse.lines() {
        let Some(payload) = line.trim_start().strip_prefix("data:") else {
            continue;
        };
        let Ok(event) = serde_json::from_str::<Value>(payload.trim()) else {
            continue;
        };
        match event.get("type").and_then(Value::as_str) {
            Some("content_block_delta") => {
                saw_stream = true;
                if let Some(delta) = event
                    .get("delta")
                    .and_then(|d| d.get("text"))
                    .and_then(Value::as_str)
                {
                    text.push_str(delta);
                }
            },
            Some("content_block_start") => {
                saw_stream = true;
                if let Some(initial) = event
                    .get("content_block")
                    .and_then(|b| b.get("text"))
                    .and_then(Value::as_str)
                {
                    text.push_str(initial);
                }
            },
            Some("message_start") => saw_stream = true,
            Some("message_stop") => complete = true,
            _ => {},
        }
    }

    saw_stream.then_some(StreamedAnswer { text, complete })
}

// Why: `complete` is false when the stored excerpt was cut off before
// `message_stop` — a property of our excerpt, not of the model's answer, so
// the caller flags it rather than scoring it down.
#[derive(Debug, Clone)]
pub(crate) struct StreamedAnswer {
    pub text: String,
    pub complete: bool,
}
