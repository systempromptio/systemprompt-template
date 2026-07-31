//! Per-trace row rendering for the Trace Explorer list.
//!
//! Maps one [`TraceSummary`] to the typed row the template iterates, including
//! the human-facing token / cost / duration formatting.

use serde::Serialize;
use systemprompt::identifiers::{AgentId, SessionId, TraceId, UserId};
use urlencoding::encode as urlencode;

use crate::handlers::ssr::format::{
    format_cost, format_duration_ms, format_token_total, short_num,
};
use crate::repositories::traces::TraceSummary;

use super::BASE_URL;

#[derive(Debug, Serialize)]
pub(super) struct TraceRow {
    session_id: SessionId,
    session_id_short: String,
    trace_id: Option<TraceId>,
    trace_id_short: Option<String>,
    started_at: String,
    started_at_time: String,
    started_at_day: String,
    active_ms: i64,
    active_display: String,
    window_display: String,
    user_id: Option<UserId>,
    user_label: String,
    agent_id: Option<AgentId>,
    agent_scope: Option<String>,
    model: Option<String>,
    model_short: Option<String>,
    provider: Option<String>,
    span_count: i64,
    request_count: i64,
    tool_call_count: i64,
    governance_count: i64,
    deny_count: i64,
    total_tokens: i64,
    tokens_display: String,
    tokens_split_display: String,
    activity_display: String,
    governance_display: String,
    cost_display: String,
    total_cost_microdollars: i64,
    cache_hit_any: bool,
    top_tool: Option<String>,
    has_error: bool,
    has_deny: bool,
    detail_url: String,
}

pub(super) fn trace_to_json(t: &TraceSummary) -> TraceRow {
    let started_local = t.started_at.with_timezone(&chrono::Local);
    TraceRow {
        session_id: t.session_id.clone(),
        session_id_short: ellipsize(t.session_id.as_str(), 14),
        trace_id: t.trace_id.clone(),
        trace_id_short: t.trace_id.as_ref().map(|id| ellipsize(id.as_str(), 12)),
        started_at: t.started_at.to_rfc3339(),
        started_at_time: started_local.format("%H:%M:%S").to_string(),
        started_at_day: started_local.format("%b %-d").to_string(),
        active_ms: t.active_ms,
        active_display: format_duration_ms(t.active_ms),
        window_display: format_duration_ms(t.window_ms),
        user_id: t.user_id.clone(),
        user_label: t.user_label.clone().unwrap_or_else(|| {
            t.user_id
                .as_ref()
                .map_or_else(|| "—".to_owned(), |id| ellipsize(id.as_str(), 12))
        }),
        agent_id: t.agent_id.clone(),
        agent_scope: t.agent_scope.clone(),
        model: t.model.clone(),
        model_short: t.model.as_deref().map(short_model),
        provider: t.provider.clone(),
        span_count: t.span_count,
        request_count: t.request_count,
        tool_call_count: t.tool_call_count,
        governance_count: t.governance_count,
        deny_count: t.deny_count,
        total_tokens: t.total_tokens,
        tokens_display: format_token_total(t.total_tokens),
        tokens_split_display: format_token_split(t.total_tokens, t.input_tokens, t.output_tokens),
        activity_display: format_requests(t.request_count),
        governance_display: format_governance(t.governance_count, t.tool_call_count),
        cost_display: format_cost(t.total_cost_microdollars),
        total_cost_microdollars: t.total_cost_microdollars,
        cache_hit_any: t.cache_hit_any,
        top_tool: t.top_tool.clone(),
        has_error: t.has_error,
        has_deny: t.has_deny,
        detail_url: format!("{BASE_URL}/{}", urlencode(t.session_id.as_str())),
    }
}

fn ellipsize(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_owned();
    }
    let head: String = s.chars().take(max).collect();
    format!("{head}…")
}

fn short_model(model: &str) -> String {
    let tail = model.rsplit('/').next().unwrap_or(model);
    let trimmed = tail
        .rsplit_once('-')
        .filter(|(_, suffix)| suffix.len() == 8 && suffix.chars().all(|c| c.is_ascii_digit()))
        .map_or(tail, |(head, _)| head);
    ellipsize(trimmed, 22)
}

fn format_requests(requests: i64) -> String {
    match requests {
        0 => "no requests".to_owned(),
        1 => "1 req".to_owned(),
        n => format!("{n} reqs"),
    }
}

fn format_governance(governance: i64, tools: i64) -> String {
    let mut parts = Vec::new();
    if governance > 0 {
        parts.push(format!("{governance} gov"));
    }
    if tools > 0 {
        parts.push(format!("{tools} tool"));
    }
    if parts.is_empty() {
        return String::new();
    }
    parts.join(" · ")
}

fn format_token_split(total: i64, input: i64, output: i64) -> String {
    if total <= 0 {
        return String::new();
    }
    format!("{} in / {} out", short_num(input), short_num(output))
}
