use serde_json::json;

use crate::types::{DECISION_DENY, POLICY_SECRET_INJECTION};

use super::queries::{SessionSummaryRow, TraceEntity, TraceEvent, TraceGovernanceRow};

pub(super) fn build_trace_data(
    session_id: &str,
    events: &[TraceEvent],
    governance: &[TraceGovernanceRow],
    entities: &[TraceEntity],
    summary: &SessionSummaryRow,
) -> serde_json::Value {
    let events_json = build_events_json(events);
    let governance_json = build_governance_json(governance);
    let entities_json = build_entities_json(entities);

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
        },
        "events": events_json,
        "has_events": !events_json.is_empty(),
        "governance": governance_json,
        "has_governance": !governance_json.is_empty(),
        "entities": entities_json,
        "has_entities": !entities_json.is_empty(),
    })
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
