//! View-model builders for the single-context detail page: map repository rows
//! into the typed template-context structs in `context`, including the
//! interleaved chronological transcript.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::repositories::analytics_grp::context_detail::{
    ContextHeader, ContextKpis, ContextMessageRow, ContextRequestRow, ContextToolCallRow,
};

use super::context::{
    ContextDetailPageContext, HeaderView, KpisView, RequestRowView, TranscriptEntryView,
};

const TRANSCRIPT_PREVIEW_CHARS: usize = 4000;

pub(super) const fn default_kpis() -> ContextKpis {
    ContextKpis {
        request_count: 0,
        trace_count: 0,
        error_count: 0,
        total_input_tokens: 0,
        total_output_tokens: 0,
        total_cost_microdollars: 0,
        first_request_at: None,
        last_request_at: None,
        model: None,
    }
}

pub(super) fn build_detail_data(
    header: &ContextHeader,
    kpis: &ContextKpis,
    requests: &[ContextRequestRow],
    messages: &[ContextMessageRow],
    tool_calls: &[ContextToolCallRow],
) -> ContextDetailPageContext {
    let transcript = build_transcript(messages, tool_calls);
    let back_url = header.session_id.as_ref().map_or_else(
        || "/admin/overview/pulse".to_owned(),
        |s| format!("/admin/sessions/{}", urlencoding::encode(s)),
    );
    let back_label = header
        .session_id
        .as_ref()
        .map_or_else(|| "Live ticker".to_owned(), |_| "Session".to_owned());
    ContextDetailPageContext {
        page: "context-detail",
        title: format!("Context · {}", short_id(&header.context_id)),
        header: header_view(header),
        kpis: kpis_view(kpis),
        has_transcript: !messages.is_empty() || !tool_calls.is_empty(),
        transcript,
        has_requests: !requests.is_empty(),
        requests: requests.iter().map(request_view).collect(),
        back_url,
        back_label,
    }
}

fn header_view(h: &ContextHeader) -> HeaderView {
    HeaderView {
        context_id: h.context_id.clone(),
        context_id_short: short_id(&h.context_id),
        user_id: h.user_id.clone(),
        user_url: h
            .user_id
            .as_ref()
            .map(|u| format!("/admin/user?id={}", urlencoding::encode(u))),
        display_name: h.display_name.clone(),
        session_id: h.session_id.clone(),
        session_url: h
            .session_id
            .as_ref()
            .map(|s| format!("/admin/sessions/{}", urlencoding::encode(s))),
        name: h.name.clone(),
        created_at: h.created_at.map(|t| t.to_rfc3339()),
        created_at_local: h.created_at.map(|t| {
            t.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }),
        updated_at_local: h.updated_at.map(|t| {
            t.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }),
    }
}

fn kpis_view(k: &ContextKpis) -> KpisView {
    KpisView {
        request_count: k.request_count,
        trace_count: k.trace_count,
        error_count: k.error_count,
        total_input_tokens: k.total_input_tokens,
        total_output_tokens: k.total_output_tokens,
        total_tokens: k.total_input_tokens + k.total_output_tokens,
        total_cost_microdollars: k.total_cost_microdollars,
        total_cost_display: format_cost(k.total_cost_microdollars),
        model: k.model.clone(),
        first_request_at_local: k.first_request_at.map(|t| {
            t.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }),
        last_request_at_local: k.last_request_at.map(|t| {
            t.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }),
    }
}

fn request_view(r: &ContextRequestRow) -> RequestRowView {
    RequestRowView {
        id: r.id.clone(),
        id_short: short_id(&r.id),
        request_url: format!("/admin/requests/{}", urlencoding::encode(&r.id)),
        trace_id: r.trace_id.clone(),
        trace_id_short: r.trace_id.as_deref().map(short_id),
        trace_url: r
            .trace_id
            .as_ref()
            .map(|t| format!("/admin/traces/{}", urlencoding::encode(t))),
        model: r.model.clone(),
        status: r.status.clone(),
        is_error: r.status == "failed",
        latency_display: r
            .latency_ms
            .map_or_else(|| "—".to_owned(), |ms| format!("{ms}ms")),
        cost_display: format_cost(r.cost_microdollars),
        created_at_local: r
            .created_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    }
}

/// Build a chronological transcript by interleaving messages and tool calls
/// within each request, then ordering requests by `created_at`.
fn build_transcript(
    messages: &[ContextMessageRow],
    tool_calls: &[ContextToolCallRow],
) -> Vec<TranscriptEntryView> {
    #[derive(Clone)]
    struct Entry {
        seq: i32,
        ts: DateTime<Utc>,
        kind: &'static str,
        role: String,
        content: String,
        tool_name: Option<String>,
        tool_input: Option<Value>,
        tool_result: Option<Value>,
    }

    let mut by_request: BTreeMap<(DateTime<Utc>, String), Vec<Entry>> = BTreeMap::new();

    for m in messages {
        by_request
            .entry((m.created_at, m.request_id.clone()))
            .or_default()
            .push(Entry {
                seq: m.sequence_number,
                ts: m.created_at,
                kind: "message",
                role: m.role.clone(),
                content: m.content.clone(),
                tool_name: None,
                tool_input: None,
                tool_result: None,
            });
    }
    for t in tool_calls {
        by_request
            .entry((t.created_at, t.request_id.clone()))
            .or_default()
            .push(Entry {
                seq: t.sequence_number,
                ts: t.created_at,
                kind: "tool_call",
                role: "tool".into(),
                content: String::new(),
                tool_name: Some(t.tool_name.clone()),
                tool_input: Some(t.tool_input.clone()),
                tool_result: t.tool_result_payload.clone(),
            });
    }

    let mut out = Vec::new();
    for ((ts, request_id), mut entries) in by_request {
        entries.sort_by_key(|e| e.seq);
        for e in entries {
            out.push(TranscriptEntryView {
                request_id: request_id.clone(),
                request_url: format!("/admin/requests/{}", urlencoding::encode(&request_id)),
                ts_local: e
                    .ts
                    .with_timezone(&chrono::Local)
                    .format("%H:%M:%S")
                    .to_string(),
                ts_full: ts.to_rfc3339(),
                kind: e.kind,
                is_user: e.role == "user",
                is_assistant: e.role == "assistant",
                is_system: e.role == "system",
                is_tool: e.kind == "tool_call",
                role: e.role,
                content_preview: preview(&e.content),
                tool_name: e.tool_name,
                tool_input_pretty: e.tool_input.as_ref().map(pretty_json),
                tool_result_pretty: e.tool_result.as_ref().map(pretty_json),
            });
        }
    }
    out
}

fn pretty_json(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
}

fn preview(s: &str) -> String {
    if s.chars().count() > TRANSCRIPT_PREVIEW_CHARS {
        let mut iter = s.chars();
        let head: String = (&mut iter).take(TRANSCRIPT_PREVIEW_CHARS).collect();
        format!("{head}…")
    } else {
        s.to_owned()
    }
}

fn short_id(id: &str) -> String {
    if id.len() > 12 {
        format!("{}…", &id[..12])
    } else {
        id.to_owned()
    }
}

fn format_cost(microdollars: i64) -> String {
    let dollars = microdollars as f64 / 1_000_000.0;
    if dollars == 0.0 {
        "$0".to_owned()
    } else if dollars < 0.01 {
        format!("${dollars:.6}")
    } else {
        format!("${dollars:.4}")
    }
}
