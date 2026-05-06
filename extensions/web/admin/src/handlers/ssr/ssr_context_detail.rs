//! `/admin/contexts/{context_id}` — single-context detail page.
//!
//! Renders header, KPIs, the chronological conversation transcript (every
//! user/assistant message + tool call interleaved by request and sequence),
//! and the request rollup. Mirrors `core contexts show` plus message detail.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::repositories::analytics_grp::context_detail::{
    fetch_context_header, fetch_context_kpis, fetch_context_messages, fetch_context_requests,
    fetch_context_tool_calls, ContextHeader, ContextKpis, ContextMessageRow, ContextRequestRow,
    ContextToolCallRow,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const NOT_FOUND_HTML: &str = "<h1>Context not found</h1>\
<p>No context, AI request, or message rows match that context id.</p>";

const TRANSCRIPT_PREVIEW_CHARS: usize = 4000;

pub async fn context_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(context_id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let context_id = context_id.trim();
    if context_id.is_empty() {
        return (StatusCode::NOT_FOUND, Html(NOT_FOUND_HTML)).into_response();
    }

    let header = match fetch_context_header(&pool, context_id).await {
        Ok(Some(h)) => h,
        Ok(None) => return (StatusCode::NOT_FOUND, Html(NOT_FOUND_HTML)).into_response(),
        Err(e) => {
            tracing::error!(error = %e, context_id, "fetch_context_header failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, Html(NOT_FOUND_HTML)).into_response();
        }
    };

    let (kpis_res, requests_res, messages_res, tool_calls_res) = tokio::join!(
        fetch_context_kpis(&pool, context_id),
        fetch_context_requests(&pool, context_id),
        fetch_context_messages(&pool, context_id),
        fetch_context_tool_calls(&pool, context_id),
    );

    let kpis = kpis_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_context_kpis failed");
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
    });
    let requests = requests_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_context_requests failed");
        Vec::new()
    });
    let messages = messages_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_context_messages failed");
        Vec::new()
    });
    let tool_calls = tool_calls_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_context_tool_calls failed");
        Vec::new()
    });

    let transcript = build_transcript(&messages, &tool_calls);

    let data = json!({
        "page": "context-detail",
        "title": format!("Context · {}", short_id(&header.context_id)),
        "header": header_json(&header),
        "kpis": kpis_json(&kpis),
        "transcript": transcript,
        "has_transcript": !messages.is_empty() || !tool_calls.is_empty(),
        "requests": requests.iter().map(request_json).collect::<Vec<_>>(),
        "has_requests": !requests.is_empty(),
        "back_url": header
            .session_id
            .as_ref()
            .map_or_else(|| "/admin/overview/pulse".to_string(),
                         |s| format!("/admin/sessions/{}", urlencoding::encode(s))),
        "back_label": header
            .session_id
            .as_ref()
            .map_or_else(|| "Live ticker".to_string(), |_| "Session".to_string()),
    });

    super::render_page(&engine, "context-detail", &data, &user_ctx, &mkt_ctx)
}

fn header_json(h: &ContextHeader) -> Value {
    json!({
        "context_id": h.context_id,
        "context_id_short": short_id(&h.context_id),
        "user_id": h.user_id,
        "user_url": h.user_id.as_ref().map(|u| format!("/admin/user?id={}", urlencoding::encode(u))),
        "display_name": h.display_name,
        "session_id": h.session_id,
        "session_url": h.session_id.as_ref().map(|s| format!("/admin/sessions/{}", urlencoding::encode(s))),
        "name": h.name,
        "created_at": h.created_at.map(|t| t.to_rfc3339()),
        "created_at_local": h.created_at.map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string()),
        "updated_at_local": h.updated_at.map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string()),
    })
}

fn kpis_json(k: &ContextKpis) -> Value {
    json!({
        "request_count": k.request_count,
        "trace_count": k.trace_count,
        "error_count": k.error_count,
        "total_input_tokens": k.total_input_tokens,
        "total_output_tokens": k.total_output_tokens,
        "total_tokens": k.total_input_tokens + k.total_output_tokens,
        "total_cost_microdollars": k.total_cost_microdollars,
        "total_cost_display": format_cost(k.total_cost_microdollars),
        "model": k.model,
        "first_request_at_local": k.first_request_at.map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string()),
        "last_request_at_local": k.last_request_at.map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string()),
    })
}

fn request_json(r: &ContextRequestRow) -> Value {
    json!({
        "id": r.id,
        "id_short": short_id(&r.id),
        "request_url": format!("/admin/requests/{}", urlencoding::encode(&r.id)),
        "trace_id": r.trace_id,
        "trace_id_short": r.trace_id.as_deref().map(short_id),
        "trace_url": r.trace_id.as_ref().map(|t| format!("/admin/traces/{}", urlencoding::encode(t))),
        "model": r.model,
        "status": r.status,
        "is_error": r.status == "failed",
        "latency_display": r.latency_ms.map_or_else(|| "—".to_string(), |ms| format!("{ms}ms")),
        "cost_display": format_cost(r.cost_microdollars),
        "created_at_local": r.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}

/// Build a chronological transcript by interleaving messages and tool calls
/// within each request, then ordering requests by `created_at`.
fn build_transcript(
    messages: &[ContextMessageRow],
    tool_calls: &[ContextToolCallRow],
) -> Vec<Value> {
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
            out.push(json!({
                "request_id": request_id,
                "request_url": format!("/admin/requests/{}", urlencoding::encode(&request_id)),
                "ts_local": e.ts.with_timezone(&chrono::Local).format("%H:%M:%S").to_string(),
                "ts_full": ts.to_rfc3339(),
                "kind": e.kind,
                "role": e.role,
                "is_user": e.role == "user",
                "is_assistant": e.role == "assistant",
                "is_system": e.role == "system",
                "is_tool": e.kind == "tool_call",
                "content_preview": preview(&e.content),
                "tool_name": e.tool_name,
                "tool_input_pretty": e.tool_input.as_ref().map(pretty_json),
                "tool_result_pretty": e.tool_result.as_ref().map(pretty_json),
            }));
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
        s.to_string()
    }
}

fn short_id(id: &str) -> String {
    if id.len() > 12 {
        format!("{}…", &id[..12])
    } else {
        id.to_string()
    }
}

fn format_cost(microdollars: i64) -> String {
    let dollars = microdollars as f64 / 1_000_000.0;
    if dollars == 0.0 {
        "$0".to_string()
    } else if dollars < 0.01 {
        format!("${dollars:.6}")
    } else {
        format!("${dollars:.4}")
    }
}
