//! The title bar and stat strip both conversation readers share — the admin
//! context view and the owner-facing history page render the same
//! `ConversationView`, and they resolve the same title, timeline and numbers
//! from the same header and KPI rows. Stated once here so the two pages cannot
//! name one conversation two different ways.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::handlers::ssr::format::{format_cost, format_span, format_token_total, short_id};
use crate::handlers::ssr::transcript_view::ConversationView;
use crate::repositories::analytics::context_detail::{ContextHeader, ContextKpis};

const TITLE_PROMPT_CHARS: usize = 120;
const GATEWAY_DEFAULT_NAME: &str = "Gateway conversation";

#[derive(Debug, Serialize)]
pub(crate) struct ConversationStatsView {
    pub turn_count: i64,
    pub tool_call_count: i64,
    pub tokens_display: String,
    pub tokens_note: String,
    pub cost_display: String,
    pub error_count: i64,
    pub side_call_count: i64,
    pub side_call_cost_display: String,
    pub models: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct StatusBadgeView {
    pub text: String,
    pub tone: &'static str,
}

// Why: the hooks pipeline's own title beats the context name, which is the
// gateway's placeholder for most rows; the first prompt beats a bare id.
pub(crate) fn resolve_title(header: &ContextHeader, view: &ConversationView) -> String {
    if let Some(t) = header
        .ai_title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        return t.to_owned();
    }
    if let Some(n) = header
        .name
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty() && *n != GATEWAY_DEFAULT_NAME)
    {
        return n.to_owned();
    }
    if let Some(prompt) = first_prompt(view) {
        return prompt;
    }
    format!("Conversation {}", short_id(header.context_id.as_str()))
}

fn first_prompt(view: &ConversationView) -> Option<String> {
    let turn = view.threads.iter().flat_map(|t| t.turns.iter()).next()?;
    let collapsed = turn.prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return None;
    }
    let mut chars = collapsed.chars();
    let head: String = chars.by_ref().take(TITLE_PROMPT_CHARS).collect();
    Some(if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    })
}

// Why: "Mon 8 Sep, 12:02 · 25 min" — when it started and how long it ran,
// which is what a reader scanning a list of conversations asked for; the full
// timestamps sit in the ids row for anyone who needs the second.
pub(crate) fn timeline_display(
    first: Option<DateTime<Utc>>,
    last: Option<DateTime<Utc>>,
) -> Option<String> {
    let start = first?;
    let started = start
        .with_timezone(&chrono::Local)
        .format("%a %-d %b, %H:%M")
        .to_string();
    Some(match last {
        Some(end) if end > start => format!("{started} · {}", format_span(first, last)),
        _ => started,
    })
}

pub(crate) fn stats_view(kpis: &ContextKpis) -> ConversationStatsView {
    ConversationStatsView {
        turn_count: kpis.turn_count,
        tool_call_count: kpis.tool_call_count,
        tokens_display: format_token_total(kpis.total_input_tokens + kpis.total_output_tokens),
        tokens_note: format!(
            "{} in / {} out",
            format_token_total(kpis.total_input_tokens),
            format_token_total(kpis.total_output_tokens)
        ),
        cost_display: format_cost(kpis.total_cost_microdollars),
        error_count: kpis.error_count,
        side_call_count: kpis.side_call_count,
        side_call_cost_display: format_cost(kpis.side_call_cost_microdollars),
        models: kpis.models.clone(),
    }
}

pub(crate) fn status_badge(hook_status: Option<&str>, error_count: i64) -> Option<StatusBadgeView> {
    let status = hook_status.map(str::trim).filter(|s| !s.is_empty());
    let text = match status {
        Some(s) => s.to_owned(),
        None if error_count > 0 => "errors".to_owned(),
        None => return None,
    };
    let tone = match text.as_str() {
        "active" | "running" | "in_progress" => "ok",
        "failed" | "error" | "errors" => "err",
        "completed" | "ended" | "done" => "info",
        _ => "muted",
    };
    Some(StatusBadgeView { text, tone })
}
