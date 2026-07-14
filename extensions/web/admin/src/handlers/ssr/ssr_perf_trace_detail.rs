//! `/admin/entities/traces/{id}` — Per-trace waterfall page.
//!
//! `id` may be a `session_id` or a `trace_id`. The handler resolves it to a
//! session id, fetches all spans, and renders a waterfall whose bars are
//! color-coded by `kind`. Each bar carries `data-chain-id` so clicks open
//! the chain-drawer focused on that span.

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde_json::json;
use sqlx::PgPool;

use crate::repositories::perf_grp::traces::{
    Span, SpanStatus, fetch_trace_spans, resolve_trace_session,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const NOT_FOUND_HTML: &str = "<h1>Trace not found</h1>\
<p>No spans found for that session or trace id.</p>";

pub(crate) async fn perf_trace_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let session_id = match resolve_trace_session(&pool, &id).await {
        Ok(Some(s)) => s,
        Ok(None) => return (StatusCode::NOT_FOUND, Html(NOT_FOUND_HTML)).into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "resolve_trace_session failed");
            return (StatusCode::NOT_FOUND, Html(NOT_FOUND_HTML)).into_response();
        },
    };

    let spans = fetch_trace_spans(&pool, &session_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "fetch_trace_spans failed");
            Vec::new()
        });

    if spans.is_empty() {
        return (StatusCode::NOT_FOUND, Html(NOT_FOUND_HTML)).into_response();
    }

    let summary = build_summary(&session_id, &spans);
    let spans_json = spans.iter().map(span_to_json).collect::<Vec<_>>();
    let spans_payload = serde_json::to_string(&spans_json).unwrap_or_else(|_| "[]".to_owned());

    let data = json!({
        "page": "trace-detail",
        "title": format!("Trace · {}", short_id(&session_id)),
        "summary": summary,
        "spans": spans_json,
        "spans_payload": spans_payload,
        "back_url": "/admin/entities/traces",
    });

    super::render_page(&engine, "perf-trace-detail", &data, &user_ctx, &mkt_ctx)
}

fn build_summary(session_id: &str, spans: &[Span]) -> serde_json::Value {
    let started = spans.iter().map(|s| s.started_at).min();
    let ended = spans.iter().map(|s| s.ended_at).max();
    let total_ms = match (started, ended) {
        (Some(a), Some(b)) => (b - a).num_milliseconds().max(0),
        _ => 0,
    };
    let identity = spans
        .iter()
        .find_map(|s| s.identity_label.clone())
        .unwrap_or_else(|| "unknown".to_owned());
    let span_count = spans.len();
    let deny_count = spans
        .iter()
        .filter(|s| matches!(s.status, SpanStatus::Deny))
        .count();
    let error_count = spans
        .iter()
        .filter(|s| matches!(s.status, SpanStatus::Error))
        .count();
    json!({
        "session_id": session_id,
        "session_id_short": short_id(session_id),
        "started_at": started.map(|t| t.to_rfc3339()),
        "started_at_local": started
            .map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string()),
        "ended_at": ended.map(|t| t.to_rfc3339()),
        "duration_ms": total_ms,
        "duration_display": format_duration(total_ms),
        "identity": identity,
        "span_count": span_count,
        "deny_count": deny_count,
        "error_count": error_count,
    })
}

fn span_to_json(s: &Span) -> serde_json::Value {
    json!({
        "id": s.id,
        "kind": s.kind.as_str(),
        "name": s.name,
        "started_at": s.started_at.to_rfc3339(),
        "ended_at": s.ended_at.to_rfc3339(),
        "duration_ms": s.duration_ms,
        "status": s.status.as_str(),
        "identity_label": s.identity_label,
        "raw": s.raw,
    })
}

fn short_id(id: &str) -> String {
    if id.len() > 12 {
        format!("{}…", &id[..12])
    } else {
        id.to_owned()
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
