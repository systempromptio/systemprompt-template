//! Per-trace row rendering for the Trace Explorer list.
//!
//! Maps one [`TraceSummary`] to the serde row the template iterates, including
//! the human-facing token / cost / duration formatting.

use serde_json::json;
use urlencoding::encode as urlencode;

use crate::repositories::perf_grp::traces::TraceSummary;

use super::BASE_URL;

pub(super) fn trace_to_json(t: &TraceSummary) -> serde_json::Value {
    let short = if t.session_id.len() > 12 {
        format!("{}…", &t.session_id[..12])
    } else {
        t.session_id.clone()
    };
    json!({
        "session_id": t.session_id,
        "session_id_short": short,
        "trace_id": t.trace_id,
        "started_at": t.started_at.to_rfc3339(),
        "started_at_local": t
            .started_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        "duration_ms": t.duration_ms,
        "duration_display": format_duration(t.duration_ms),
        "user_id": t.user_id,
        "agent_id": t.agent_id,
        "agent_scope": t.agent_scope,
        "model": t.model,
        "provider": t.provider,
        "span_count": t.span_count,
        "request_count": t.request_count,
        "tool_call_count": t.tool_call_count,
        "governance_count": t.governance_count,
        "deny_count": t.deny_count,
        "total_tokens": t.total_tokens,
        "tokens_display": format_tokens(t.total_tokens, t.input_tokens, t.output_tokens),
        "cost_display": format_cost(t.total_cost_microdollars),
        "total_cost_microdollars": t.total_cost_microdollars,
        "latency_display": format_duration(t.total_latency_ms),
        "cache_hit_any": t.cache_hit_any,
        "top_tool": t.top_tool,
        "has_error": t.has_error,
        "has_deny": t.has_deny,
        "detail_url": format!("{BASE_URL}/{}", urlencode(&t.session_id)),
    })
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
