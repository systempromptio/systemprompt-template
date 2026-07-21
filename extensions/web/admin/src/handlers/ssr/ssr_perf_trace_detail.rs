//! `/admin/entities/traces/{id}` — Per-trace waterfall page.
//!
//! `id` may be a `session_id` or a `trace_id`. The handler resolves it to a
//! session id, fetches all spans, and renders a waterfall whose bars are
//! color-coded by `kind`. Each bar carries `data-chain-id` so clicks open
//! the chain-drawer focused on that span.

use crate::error::AdminError;
use std::sync::Arc;

use systemprompt::identifiers::SessionId;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;

use crate::error::AdminHtmlResult;
use crate::repositories::traces::{Span, SpanStatus, list_trace_spans, resolve_trace_session};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};


#[derive(Debug, Serialize)]
struct TraceDetailContext {
    page: &'static str,
    title: String,
    summary: Summary,
    spans: Vec<Span>,
    spans_payload: String,
    back_url: &'static str,
}

#[derive(Debug, Serialize)]
struct Summary {
    session_id: SessionId,
    session_id_short: String,
    started_at: Option<String>,
    started_at_local: Option<String>,
    ended_at: Option<String>,
    duration_ms: i64,
    duration_display: String,
    identity: String,
    span_count: usize,
    deny_count: usize,
    error_count: usize,
}

pub(crate) async fn perf_trace_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let Some(session_id) = resolve_trace_session(&pool, &id).await? else {
        return Err(AdminError::NotFound(
            "No spans found for that session or trace id.".to_owned(),
        )
        .into());
    };

    // `resolve_trace_session` just succeeded, which proves rows exist for this
    // id. Degrading a failed span fetch to an empty vec would fall into the
    // not-found arm below and answer "No spans found for that session or trace
    // id" — asserting as fact something this handler disproved one statement
    // earlier, and sending an investigator away believing the trace was purged.
    let spans = list_trace_spans(&pool, &session_id).await?;

    if spans.is_empty() {
        return Err(AdminError::NotFound(
            "No spans found for that session or trace id.".to_owned(),
        )
        .into());
    }

    let summary = build_summary(&session_id, &spans);
    let spans_payload = serde_json::to_string(&spans).unwrap_or_else(|_| "[]".to_owned());

    let ctx = TraceDetailContext {
        page: "trace-detail",
        title: format!("Trace · {}", short_id(session_id.as_str())),
        summary,
        spans,
        spans_payload,
        back_url: "/admin/entities/traces",
    };

    Ok(super::render_typed_page(
        &engine,
        "perf-trace-detail",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn build_summary(session_id: &SessionId, spans: &[Span]) -> Summary {
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
    Summary {
        session_id: session_id.clone(),
        session_id_short: short_id(session_id.as_str()),
        started_at: started.map(|t| t.to_rfc3339()),
        started_at_local: started.map(|t| {
            t.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }),
        ended_at: ended.map(|t| t.to_rfc3339()),
        duration_ms: total_ms,
        duration_display: format_duration(total_ms),
        identity,
        span_count,
        deny_count,
        error_count,
    }
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
