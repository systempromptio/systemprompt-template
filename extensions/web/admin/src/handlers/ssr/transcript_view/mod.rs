//! The transcript view-model, shared by the admin context-detail page and the
//! owner-facing conversation page under `/admin/history`.
//!
//! [`build_conversation`] is the reader's model: one turn per human prompt,
//! assembled from the latest request's stored history so a ten-turn
//! conversation renders ten turns rather than the fifty-five messages the
//! gateway persisted. The two pages differ only in how text is treated on the
//! way out, which is what [`TranscriptOptions`] carries: the owner-facing page
//! strips the gateway's `=== USER PROMPT ===` framing and runs every body
//! through the credential redactor, the admin page shows the stored text as
//! it is.

use serde::Serialize;
// JSON: per-tool payloads pretty-printed for the transcript view.
use serde_json::Value;

use crate::handlers::ssr::format::format_cost;
use crate::repositories::analytics::context_detail::ContextRequestRow;
use crate::repositories::analytics::conversations::{redact_text, strip_gateway_markers};

mod conversation;
mod markers;
mod thread;

pub use conversation::{
    ConversationView, SideCallRowView, SideCallsView, StepView, ThreadView, ToolChipView, TurnView,
    build_conversation,
};
pub use markers::{
    ParsedAssistant, ToolUseMarker, parse_assistant, strip_system_reminders, tidy_lines,
};

const TRANSCRIPT_PREVIEW_CHARS: usize = 4000;

// Why: The per-turn telemetry rail: how the turn was served and what it cost.
// Cloned onto every turn of the same request, so all of a request's messages
// state the same numbers rather than only the last one.
#[derive(Debug, Clone, Serialize)]
pub struct TranscriptMetaView {
    pub model: String,
    pub status: String,
    pub is_error: bool,
    pub latency_display: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_display: Option<String>,
    pub cost_display: String,
}

// Why: how message bodies are treated on the way into the view.
#[derive(Debug, Clone, Copy, Default)]
pub struct TranscriptOptions {
    // Why: cut the gateway's `=== USER PROMPT ===` /
    // Why: `=== ASSISTANT ANSWER ===` framing out of user turns before display.
    pub strip_markers: bool,
    // Why: run every body through the credential redactor.
    pub redact: bool,
}

impl TranscriptOptions {
    // Why: what the owner-facing conversation page uses.
    pub const fn owner_facing() -> Self {
        Self {
            strip_markers: true,
            redact: true,
        }
    }
}

pub(crate) fn display_body(content: &str, opts: TranscriptOptions) -> (String, u32) {
    let stripped = if opts.strip_markers {
        strip_gateway_markers(content)
    } else {
        content.to_owned()
    };
    if opts.redact {
        redact_text(&stripped)
    } else {
        (stripped, 0)
    }
}

pub fn meta_view(r: &ContextRequestRow) -> TranscriptMetaView {
    TranscriptMetaView {
        model: r.model.clone().unwrap_or_else(|| "—".to_owned()),
        status: r.status.clone(),
        is_error: r.status == "failed",
        latency_display: format_latency(r.latency_ms),
        token_display: format_tokens(r.input_tokens, r.output_tokens),
        cost_display: format_cost(r.cost_microdollars),
    }
}

pub(crate) fn format_latency(latency_ms: Option<i32>) -> String {
    latency_ms.map_or_else(|| "—".to_owned(), |ms| format!("{ms}ms"))
}

pub(crate) fn format_tokens(input: Option<i32>, output: Option<i32>) -> Option<String> {
    match (input, output) {
        (None, None) => None,
        (i, o) => Some(format!(
            "{} in / {} out",
            i.unwrap_or_default(),
            o.unwrap_or_default()
        )),
    }
}

// JSON: pretty-prints the per-tool payloads above for the transcript view
pub(crate) fn pretty_json(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
}

pub fn preview(s: &str) -> String {
    if s.chars().count() > TRANSCRIPT_PREVIEW_CHARS {
        let mut iter = s.chars();
        let head: String = (&mut iter).take(TRANSCRIPT_PREVIEW_CHARS).collect();
        format!("{head}…")
    } else {
        s.to_owned()
    }
}

pub fn short_id(id: &str) -> String {
    if id.len() > 12 {
        format!("{}…", &id[..12])
    } else {
        id.to_owned()
    }
}
