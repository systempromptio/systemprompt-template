use serde_json::json;

use crate::types::{DECISION_DENY, POLICY_SECRET_INJECTION};

use super::queries::{
    AiCallRow, AiMessageRow, AiToolCallRow, SessionSummaryRow, TraceEntity, TraceEvent,
    TraceGovernanceRow,
};

const PREVIEW_CHARS: usize = 4000;

#[allow(clippy::too_many_arguments)]
pub(super) fn build_trace_data(
    session_id: &str,
    events: &[TraceEvent],
    governance: &[TraceGovernanceRow],
    entities: &[TraceEntity],
    summary: &SessionSummaryRow,
    ai_calls: &[AiCallRow],
    ai_messages: &[AiMessageRow],
    ai_tool_calls: &[AiToolCallRow],
) -> serde_json::Value {
    let events_json = build_events_json(events);
    let governance_json = build_governance_json(governance);
    let entities_json = build_entities_json(entities);
    let ai_conversations_json = build_ai_conversations_json(ai_calls, ai_messages, ai_tool_calls);

    let duration_ms = events
        .first()
        .zip(events.last())
        .map_or(0, |(first, last)| {
            (last.created_at - first.created_at).num_milliseconds()
        });

    json!({
        "page": "traces",
        "title": format!("Trace — {}", &session_id[..session_id.len().min(12)]),
        "has_session": true,
        "session_id": session_id,
        "summary": {
            "total_events": summary.total_events,
            "tool_uses": summary.tool_uses,
            "prompts": summary.prompts,
            "errors": summary.errors,
            "duration_ms": duration_ms,
            "governance_decisions": governance.len(),
            "ai_calls": ai_calls.len(),
        },
        "events": events_json,
        "has_events": !events_json.is_empty(),
        "governance": governance_json,
        "has_governance": !governance_json.is_empty(),
        "entities": entities_json,
        "has_entities": !entities_json.is_empty(),
        "ai_conversations": ai_conversations_json,
        "has_ai_conversations": !ai_conversations_json.is_empty(),
    })
}

fn truncate_preview(s: &str) -> (String, bool) {
    if s.chars().count() > PREVIEW_CHARS {
        let head: String = s.chars().take(PREVIEW_CHARS).collect();
        (head, true)
    } else {
        (s.to_string(), false)
    }
}

fn role_badge_class(role: &str) -> &'static str {
    match role {
        "system" => "mcp-badge-warning",
        "user" => "mcp-badge-info",
        "assistant" => "badge-purple",
        "tool" => "mcp-badge-success",
        _ => "mcp-badge-neutral",
    }
}

fn status_badge_class(status: &str) -> &'static str {
    match status {
        "completed" | "ok" | "success" => "mcp-badge-success",
        "pending" | "streaming" => "mcp-badge-info",
        _ => "mcp-badge-danger",
    }
}

fn build_ai_conversations_json(
    calls: &[AiCallRow],
    messages: &[AiMessageRow],
    tool_calls: &[AiToolCallRow],
) -> Vec<serde_json::Value> {
    calls
        .iter()
        .enumerate()
        .map(|(idx, c)| {
            let msgs: Vec<serde_json::Value> = messages
                .iter()
                .filter(|m| m.request_id == c.request_id)
                .map(|m| {
                    let (preview, truncated) = truncate_preview(&m.content);
                    json!({
                        "role": m.role,
                        "role_badge_class": role_badge_class(&m.role),
                        "sequence_number": m.sequence_number,
                        "content_preview": preview,
                        "is_truncated": truncated,
                        "content_length": m.content.len(),
                    })
                })
                .collect();
            let tcalls: Vec<serde_json::Value> = tool_calls
                .iter()
                .filter(|t| t.request_id == c.request_id)
                .map(|t| {
                    let (input_preview, input_truncated) = truncate_preview(&t.tool_input);
                    json!({
                        "tool_name": t.tool_name,
                        "sequence_number": t.sequence_number,
                        "tool_input_preview": input_preview,
                        "is_input_truncated": input_truncated,
                        "tool_result_payload": t.tool_result_payload,
                        "has_result": t.tool_result_payload.is_some(),
                        "mcp_execution_id": t.mcp_execution_id,
                    })
                })
                .collect();
            let cost_usd = c.cost_microdollars as f64 / 1_000_000.0;
            json!({
                "index": idx + 1,
                "request_id": c.request_id,
                "request_id_short": short_id(&c.request_id),
                "model": c.model,
                "provider": c.provider,
                "status": c.status,
                "status_badge_class": status_badge_class(&c.status),
                "latency_ms": c.latency_ms,
                "input_tokens": c.input_tokens,
                "output_tokens": c.output_tokens,
                "total_tokens": c.input_tokens.unwrap_or(0) + c.output_tokens.unwrap_or(0),
                "cost_display": format!("${:.6}", cost_usd),
                "trace_id": c.trace_id,
                "trace_id_short": c.trace_id.as_deref().map(short_id),
                "created_at": c.created_at.with_timezone(&chrono::Local).format("%H:%M:%S%.3f").to_string(),
                "messages": msgs,
                "message_count": messages.iter().filter(|m| m.request_id == c.request_id).count(),
                "tool_calls": tcalls,
                "tool_call_count": tool_calls.iter().filter(|t| t.request_id == c.request_id).count(),
            })
        })
        .collect()
}

fn short_id(id: &str) -> String {
    if id.len() > 12 {
        format!("{}…", &id[..12])
    } else {
        id.to_string()
    }
}

fn event_badge_class(event_type: &str) -> &'static str {
    match event_type {
        t if t.contains("ToolUse") => "mcp-badge-info",
        t if t.contains("Failure") || t.contains("Error") => "mcp-badge-danger",
        t if t.contains("Prompt") || t.contains("Submit") => "mcp-badge-success",
        t if t.contains("Session") => "mcp-badge-warning",
        t if t.contains("Subagent") => "badge-purple",
        _ => "mcp-badge-neutral",
    }
}

fn build_events_json(events: &[TraceEvent]) -> Vec<serde_json::Value> {
    let first_ts = events.first().map(|e| e.created_at);
    events
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let delta_ms = if i > 0 {
                (e.created_at - events[i - 1].created_at).num_milliseconds()
            } else {
                0
            };
            let elapsed_ms = first_ts.map_or(0, |f| (e.created_at - f).num_milliseconds());
            json!({
                "id": e.id,
                "event_type": e.event_type,
                "event_type_short": e.event_type.replace("claude_code_", ""),
                "tool_name": e.tool_name,
                "has_tool": e.tool_name.is_some(),
                "metadata": e.metadata,
                "has_metadata": e.metadata != serde_json::Value::Null && e.metadata != json!({}),
                "created_at": e.created_at.with_timezone(&chrono::Local).format("%H:%M:%S%.3f").to_string(),
                "delta_ms": delta_ms,
                "elapsed_ms": elapsed_ms,
                "badge_class": event_badge_class(&e.event_type),
            })
        })
        .collect()
}

fn build_governance_json(governance: &[TraceGovernanceRow]) -> Vec<serde_json::Value> {
    governance
        .iter()
        .map(|g| {
            json!({
                "tool_name": g.tool_name,
                "agent_id": g.agent_id,
                "agent_scope": g.agent_scope,
                "decision": g.decision,
                "is_denied": g.decision == DECISION_DENY,
                "is_secret_breach": g.policy == POLICY_SECRET_INJECTION,
                "policy": g.policy,
                "reason": g.reason,
                "created_at": g.created_at.with_timezone(&chrono::Local).format("%H:%M:%S%.3f").to_string(),
            })
        })
        .collect()
}

fn build_entities_json(entities: &[TraceEntity]) -> Vec<serde_json::Value> {
    entities
        .iter()
        .map(|e| {
            let badge_class = match e.entity_type.as_str() {
                "skill" => "badge-blue",
                "agent" => "badge-purple",
                "mcp_tool" => "mcp-badge-info",
                _ => "mcp-badge-neutral",
            };
            json!({
                "entity_type": e.entity_type,
                "entity_name": e.entity_name,
                "usage_count": e.usage_count,
                "badge_class": badge_class,
            })
        })
        .collect()
}
