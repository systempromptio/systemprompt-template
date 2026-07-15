//! Transcript JSONB parsing — turns the stored `session_transcripts.transcript`
//! array into normalised `TranscriptTurn`s, extracts tool calls and body text,
//! and attaches the per-session governance decision.

use chrono::{DateTime, Utc};
use systemprompt::identifiers::SessionId;

use super::{ToolCall, TranscriptTurn, TurnGovernance, redact_text};

pub(super) struct ParseInput<'a> {
    pub(super) session_id: &'a str,
    pub(super) transcript: &'a serde_json::Value,
    pub(super) fallback_model: Option<&'a str>,
    pub(super) governance_rows: &'a [GovernanceRow],
    pub(super) fallback_trace_id: Option<&'a str>,
    pub(super) include_raw: bool,
}

pub(super) fn parse_turns(input: &ParseInput<'_>) -> Vec<TranscriptTurn> {
    let Some(arr) = input.transcript.as_array() else {
        return vec![];
    };

    arr.iter()
        .enumerate()
        .map(|(idx, entry)| {
            let ordinal = i32::try_from(idx).unwrap_or(i32::MAX);
            let role = entry
                .get("role")
                .and_then(|v| v.as_str())
                .unwrap_or("assistant")
                .to_owned();
            let ts = entry
                .get("ts")
                .or_else(|| entry.get("timestamp"))
                .and_then(|v| v.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d.with_timezone(&Utc));
            let model = entry
                .get("model")
                .and_then(|v| v.as_str())
                .map(String::from)
                .or_else(|| input.fallback_model.map(String::from));
            let latency_ms = entry
                .get("latency_ms")
                .and_then(serde_json::Value::as_i64)
                .and_then(|v| i32::try_from(v).ok());

            let raw_text = extract_content_text(entry);
            let (redacted_text, redactions_applied) = redact_text(&raw_text);
            let content_redacted = Some(redacted_text);
            let content = if input.include_raw {
                Some(raw_text)
            } else {
                None
            };

            let tool_calls = entry
                .get("tool_calls")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().map(parse_tool_call).collect())
                .unwrap_or_default();

            let governance =
                match_governance(input.governance_rows, ordinal, input.fallback_trace_id);
            let anomaly_count = entry
                .get("anomaly_count")
                .and_then(serde_json::Value::as_i64)
                .and_then(|v| i32::try_from(v).ok())
                .unwrap_or(0);

            TranscriptTurn {
                id: format!("{}:{ordinal}", input.session_id),
                session_id: SessionId::new(input.session_id),
                ordinal,
                role,
                ts,
                model,
                latency_ms,
                content_redacted,
                redactions_applied,
                content,
                tool_calls,
                governance,
                anomaly_count,
            }
        })
        .collect()
}

fn parse_tool_call(v: &serde_json::Value) -> ToolCall {
    ToolCall {
        id: v.get("id").and_then(|x| x.as_str()).map(String::from),
        name: v
            .get("name")
            .or_else(|| v.get("tool_name"))
            .and_then(|x| x.as_str())
            .unwrap_or("unknown")
            .to_owned(),
        args_json: v
            .get("args")
            .or_else(|| v.get("input"))
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        result_json: v.get("result").or_else(|| v.get("output")).cloned(),
        duration_ms: v
            .get("duration_ms")
            .and_then(serde_json::Value::as_i64)
            .and_then(|d| i32::try_from(d).ok()),
        status: v
            .get("status")
            .and_then(|x| x.as_str())
            .unwrap_or("ok")
            .to_owned(),
    }
}

/// Pull a textual representation of a transcript entry's body.
/// Accepts plain strings, Anthropic-style content arrays, or `text` fields.
pub(super) fn extract_content_text(entry: &serde_json::Value) -> String {
    if let Some(s) = entry.get("content").and_then(|v| v.as_str()) {
        return s.to_owned();
    }
    if let Some(arr) = entry.get("content").and_then(|v| v.as_array()) {
        let mut out = String::new();
        for block in arr {
            if let Some(t) = block.get("text").and_then(|v| v.as_str()) {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(t);
            }
        }
        if !out.is_empty() {
            return out;
        }
    }
    if let Some(s) = entry.get("text").and_then(|v| v.as_str()) {
        return s.to_owned();
    }
    String::new()
}

pub(super) struct GovernanceRow {
    pub(super) decision: String,
    pub(super) evaluated_rules: serde_json::Value,
}

fn match_governance(
    rows: &[GovernanceRow],
    _ordinal: i32,
    fallback_trace_id: Option<&str>,
) -> Option<TurnGovernance> {
    let row = rows
        .iter()
        .find(|r| r.decision == "deny")
        .or_else(|| rows.first())?;

    let rule_count = i32::try_from(row.evaluated_rules.as_array().map_or(0, Vec::len)).unwrap_or(0);

    Some(TurnGovernance {
        decision: row.decision.clone(),
        trace_id: fallback_trace_id.map(String::from),
        rule_count,
        redactions_applied: 0,
    })
}
