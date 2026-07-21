//! Per-trace row rendering for the Trace Explorer list.
//!
//! Maps one [`TraceSummary`] to the typed row the template iterates, including
//! the human-facing token / cost / duration formatting.

use serde::Serialize;
use systemprompt::identifiers::{AgentId, SessionId, TraceId, UserId};
use urlencoding::encode as urlencode;

use crate::repositories::traces::TraceSummary;

use super::BASE_URL;

#[derive(Debug, Serialize)]
pub(super) struct TraceRow {
    session_id: SessionId,
    session_id_short: String,
    trace_id: Option<TraceId>,
    started_at: String,
    started_at_local: String,
    duration_ms: i64,
    duration_display: String,
    user_id: Option<UserId>,
    agent_id: Option<AgentId>,
    agent_scope: Option<String>,
    model: Option<String>,
    provider: Option<String>,
    span_count: i64,
    request_count: i64,
    tool_call_count: i64,
    governance_count: i64,
    deny_count: i64,
    total_tokens: i64,
    tokens_display: String,
    cost_display: String,
    total_cost_microdollars: i64,
    latency_display: String,
    cache_hit_any: bool,
    top_tool: Option<String>,
    has_error: bool,
    has_deny: bool,
    detail_url: String,
}

pub(super) fn trace_to_json(t: &TraceSummary) -> TraceRow {
    let sid = t.session_id.as_str();
    let short = if sid.len() > 12 {
        format!("{}…", &sid[..12])
    } else {
        sid.to_owned()
    };
    TraceRow {
        session_id: t.session_id.clone(),
        session_id_short: short,
        trace_id: t.trace_id.clone(),
        started_at: t.started_at.to_rfc3339(),
        started_at_local: t
            .started_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        duration_ms: t.duration_ms,
        duration_display: format_duration(t.duration_ms),
        user_id: t.user_id.clone(),
        agent_id: t.agent_id.clone(),
        agent_scope: t.agent_scope.clone(),
        model: t.model.clone(),
        provider: t.provider.clone(),
        span_count: t.span_count,
        request_count: t.request_count,
        tool_call_count: t.tool_call_count,
        governance_count: t.governance_count,
        deny_count: t.deny_count,
        total_tokens: t.total_tokens,
        tokens_display: format_tokens(t.total_tokens, t.input_tokens, t.output_tokens),
        cost_display: format_cost(t.total_cost_microdollars),
        total_cost_microdollars: t.total_cost_microdollars,
        latency_display: format_duration(t.total_latency_ms),
        cache_hit_any: t.cache_hit_any,
        top_tool: t.top_tool.clone(),
        has_error: t.has_error,
        has_deny: t.has_deny,
        detail_url: format!("{BASE_URL}/{}", urlencode(t.session_id.as_str())),
    }
}

fn format_tokens(total: i64, input: i64, output: i64) -> String {
    if total <= 0 {
        return "—".to_owned();
    }
    format!(
        "{} ({} in / {} out)",
        short_num(total),
        short_num(input),
        short_num(output)
    )
}

fn short_num(n: i64) -> String {
    let abs = n.unsigned_abs();
    if abs >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if abs >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_cost(micros: i64) -> String {
    if micros <= 0 {
        return "—".to_owned();
    }
    let dollars = micros as f64 / 1_000_000.0;
    if dollars >= 1.0 {
        format!("${dollars:.2}")
    } else if dollars >= 0.01 {
        format!("${dollars:.4}")
    } else {
        format!("${dollars:.6}")
    }
}

fn format_duration(ms: i64) -> String {
    if ms < 1000 {
        format!("{ms} ms")
    } else if ms < 60_000 {
        format!("{:.2} s", ms as f64 / 1000.0)
    } else {
        format!("{:.1} min", ms as f64 / 60_000.0)
    }
}
