//! The conversation reader's view-model: threads of turns, each turn a human
//! prompt followed by the assistant's steps, with probes and utility calls
//! set aside as side calls.
//!
//! The gateway stores every request's full message history, so the canonical
//! transcript of a thread is the history of its LAST request; earlier requests
//! only contribute attribution — request *i* produced the assistant row at
//! index `n_i` of that history, where `n_i` is its own history length minus
//! the reply it appended. That is what links a step to its cost, latency, and
//! tool-call rows without rendering any history twice.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::identifiers::GatewayConversationId;

use super::thread::ThreadBuilder;
use super::{TranscriptMetaView, TranscriptOptions, display_body, short_id};
use crate::handlers::ssr::entity_urls::request_detail_url;
use crate::handlers::ssr::format::format_cost;
use crate::repositories::analytics::context_detail::{
    ContextMessageRow, ContextRequestRow, ContextToolCallRow,
};

pub(super) const LONG_TEXT_CHARS: usize = 1200;
pub(super) const LONG_TEXT_LINES: usize = 12;
pub(super) const PROMPT_SHORT_CHARS: usize = 60;
pub(super) const THREAD_LABEL_CHARS: usize = 80;

#[derive(Debug, Serialize)]
pub struct ConversationView {
    pub threads: Vec<ThreadView>,
    pub turn_count: usize,
    pub tool_call_count: usize,
    pub side_calls: SideCallsView,
    pub redaction_count: u32,
    pub has_content: bool,
}

#[derive(Debug, Serialize)]
pub struct ThreadView {
    pub index: usize,
    pub is_main: bool,
    pub anchor: String,
    pub label: String,
    pub started_local: String,
    pub request_count: usize,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub system_prompt_chars: usize,
    pub turns: Vec<TurnView>,
}

#[derive(Debug, Serialize)]
pub struct TurnView {
    pub number: usize,
    pub anchor: String,
    pub prompt: String,
    pub prompt_short: String,
    pub prompt_is_long: bool,
    pub ts_local: String,
    pub ts_full: String,
    pub steps: Vec<StepView>,
    pub tool_names: Vec<ToolChipView>,
}

#[derive(Debug, Serialize)]
pub struct ToolChipView {
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct StepView {
    pub is_assistant: bool,
    pub is_tool: bool,
    pub text: Option<String>,
    pub text_is_long: bool,
    pub tool_name: Option<String>,
    pub tool_input_pretty: Option<String>,
    pub tool_result_pretty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<TranscriptMetaView>,
    pub request_id_short: Option<String>,
    pub request_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SideCallsView {
    pub count: usize,
    pub cost_display: String,
    pub rows: Vec<SideCallRowView>,
}

#[derive(Debug, Serialize)]
pub struct SideCallRowView {
    pub kind: String,
    pub ts_local: String,
    pub model: String,
    pub token_display: Option<String>,
    pub cost_display: String,
    pub status: String,
    pub request_id_short: String,
    pub request_url: String,
}

// Why: the running counters every thread shares — turn numbers and anchors
// are global across threads, and one redaction tally covers the whole page.
pub(super) struct Counters {
    pub(super) opts: TranscriptOptions,
    pub(super) turn_number: usize,
    pub(super) tool_calls: usize,
    pub(super) redactions: u32,
}

impl Counters {
    pub(super) fn body(&mut self, content: &str) -> String {
        let (body, n) = display_body(content, self.opts);
        self.redactions = self.redactions.saturating_add(n);
        body
    }
}

#[must_use]
pub fn build_conversation(
    messages: &[ContextMessageRow],
    tool_calls: &[ContextToolCallRow],
    requests: &[ContextRequestRow],
    opts: TranscriptOptions,
) -> ConversationView {
    let mut side: Vec<&ContextRequestRow> = requests
        .iter()
        .filter(|r| r.effective_kind != "turn")
        .collect();
    side.sort_by_key(|r| r.created_at);
    let side_calls = side_calls_view(&side);

    let mut turns: Vec<&ContextRequestRow> = requests
        .iter()
        .filter(|r| r.effective_kind == "turn")
        .collect();
    turns.sort_by_key(|r| r.created_at);

    let mut groups: Vec<Vec<&ContextRequestRow>> = Vec::new();
    let mut group_of: HashMap<Option<&str>, usize> = HashMap::new();
    for r in turns {
        let key = r
            .gateway_conversation_id
            .as_ref()
            .map(GatewayConversationId::as_str);
        let idx = *group_of.entry(key).or_insert_with(|| {
            groups.push(Vec::new());
            groups.len() - 1
        });
        groups[idx].push(r);
    }

    let mut messages_by_request: HashMap<&str, Vec<&ContextMessageRow>> = HashMap::new();
    for m in messages {
        messages_by_request
            .entry(m.request_id.as_str())
            .or_default()
            .push(m);
    }
    for rows in messages_by_request.values_mut() {
        rows.sort_by_key(|m| m.sequence_number);
    }
    let mut tools_by_request: HashMap<&str, Vec<&ContextToolCallRow>> = HashMap::new();
    for t in tool_calls {
        tools_by_request
            .entry(t.request_id.as_str())
            .or_default()
            .push(t);
    }
    for rows in tools_by_request.values_mut() {
        rows.sort_by_key(|t| t.sequence_number);
    }

    let mut counters = Counters {
        opts,
        turn_number: 0,
        tool_calls: 0,
        redactions: 0,
    };
    let threads: Vec<ThreadView> = groups
        .iter()
        .enumerate()
        .map(|(i, reqs)| {
            ThreadBuilder::new(&mut counters, &messages_by_request, &tools_by_request)
                .build(i + 1, reqs)
        })
        .collect();

    let has_content = threads.iter().any(|t| !t.turns.is_empty());
    ConversationView {
        threads,
        turn_count: counters.turn_number,
        tool_call_count: counters.tool_calls,
        side_calls,
        redaction_count: counters.redactions,
        has_content,
    }
}

fn side_calls_view(side: &[&ContextRequestRow]) -> SideCallsView {
    let cost: i64 = side.iter().map(|r| r.cost_microdollars).sum();
    SideCallsView {
        count: side.len(),
        cost_display: format_cost(cost),
        rows: side
            .iter()
            .map(|r| SideCallRowView {
                kind: r.effective_kind.clone(),
                ts_local: local_time(r.created_at),
                model: r.model.clone().unwrap_or_else(|| "—".to_owned()),
                token_display: super::format_tokens(r.input_tokens, r.output_tokens),
                cost_display: format_cost(r.cost_microdollars),
                status: r.status.clone(),
                request_id_short: short_id(r.id.as_str()),
                request_url: request_detail_url(&r.id),
            })
            .collect(),
    }
}

pub(super) fn local_time(ts: DateTime<Utc>) -> String {
    ts.with_timezone(&chrono::Local)
        .format("%H:%M:%S")
        .to_string()
}

pub(super) fn is_long(text: &str) -> bool {
    text.chars().count() > LONG_TEXT_CHARS || text.lines().count() > LONG_TEXT_LINES
}

// Why: one line, whitespace collapsed, cut at `max` chars with an ellipsis —
// the shape a nav rail or thread tab can hold.
pub(super) fn single_line(text: &str, max: usize) -> String {
    let collapsed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() > max {
        let head: String = collapsed.chars().take(max).collect();
        format!("{}…", head.trim_end())
    } else {
        collapsed
    }
}
