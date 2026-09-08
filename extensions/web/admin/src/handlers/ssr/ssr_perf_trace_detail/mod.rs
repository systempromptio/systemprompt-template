//! `/admin/traces/{id}` — per-trace waterfall page.
//!
//! `id` may be a `session_id` or a `trace_id`. The handler resolves it to a
//! session id, fetches every span, and renders them twice: once as an inline
//! SVG waterfall (see [`waterfall`]) and once as a table carrying the
//! provider, model, cost and latency each span reported. Every bar and every
//! row carries `data-chain-id`, so a click opens the chain drawer on that span.

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::entity_urls::session_detail_url;
use crate::handlers::ssr::format::{format_cost, format_duration_ms, local_time, short_id};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::traces::{Span, SpanStatus, list_trace_spans, resolve_trace_session};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

mod waterfall;

use waterfall::{WaterfallView, build_waterfall};

#[derive(Debug, Serialize)]
struct TraceDetailContext {
    page: &'static str,
    title: String,
    breadcrumbs: Vec<BreadcrumbView>,
    summary: Summary,
    waterfall: WaterfallView,
    spans: Vec<SpanView>,
    span_count: usize,
    back_url: &'static str,
}

#[derive(Debug, Serialize)]
struct SpanView {
    id: String,
    kind: &'static str,
    name: String,
    started_at_local: String,
    duration_display: String,
    status: &'static str,
    offset_label: String,
    provider: String,
    model: String,
    cost_display: String,
    latency_display: String,
}

#[derive(Debug, Serialize)]
struct Summary {
    session_id: SessionId,
    duration_ms: i64,
    session_id_short: String,
    session_url: String,
    started_at: Option<String>,
    started_at_local: Option<String>,
    duration_display: String,
    identity: String,
    span_count: usize,
    deny_count: usize,
    error_count: usize,
    rejected_count: usize,
    request_count: usize,
    cost_display: String,
}

pub(crate) async fn perf_trace_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let Some(session_id) = resolve_trace_session(&pool, &id).await? else {
        return Err(AdminError::NotFound(
            "No spans found for that session or trace id.".to_owned(),
        )
        .into());
    };

    // Why: `resolve_trace_session` just succeeded, which proves rows exist for
    // this id. Degrading a failed span fetch to an empty vec would fall into
    // the not-found arm below and answer "No spans found for that session or
    // trace id" — asserting as fact something this handler disproved one
    // statement earlier, and sending an investigator away believing the trace
    // was purged.
    let spans = list_trace_spans(&pool, &session_id).await?;

    if spans.is_empty() {
        return Err(AdminError::NotFound(
            "No spans found for that session or trace id.".to_owned(),
        )
        .into());
    }

    let summary = build_summary(&session_id, &spans);
    let span_views = build_span_views(&spans);

    let ctx = TraceDetailContext {
        page: "trace-detail",
        title: format!("Trace · {}", short_id(session_id.as_str())),
        breadcrumbs: breadcrumbs(&session_id),
        waterfall: build_waterfall(&spans, summary.total_ms()),
        span_count: span_views.len(),
        summary,
        spans: span_views,
        back_url: "/admin/traces",
    };

    Ok(super::render_typed_page(
        &engine,
        "perf-trace-detail",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn breadcrumbs(session_id: &SessionId) -> Vec<BreadcrumbView> {
    vec![
        BreadcrumbView::link("Traces", "/admin/traces"),
        BreadcrumbView::current(short_id(session_id.as_str())),
    ]
}

impl Summary {
    // Why: the waterfall scales to the same first-to-last window the summary
    // reports, so the two cannot disagree about how long the trace took.
    const fn total_ms(&self) -> i64 {
        self.duration_ms
    }
}

fn build_summary(session_id: &SessionId, spans: &[Span]) -> Summary {
    let started = spans.iter().map(|s| s.started_at).min();
    let ended = spans.iter().map(|s| s.ended_at).max();
    let total_ms = match (started, ended) {
        (Some(a), Some(b)) => (b - a).num_milliseconds().max(0),
        _ => 0,
    };
    let count = |want: SpanStatus| {
        spans
            .iter()
            .filter(|s| s.status.as_str() == want.as_str())
            .count()
    };
    Summary {
        session_url: session_detail_url(session_id),
        session_id: session_id.clone(),
        session_id_short: short_id(session_id.as_str()),
        started_at: started.map(|t| t.to_rfc3339()),
        started_at_local: started.map(local_time),
        duration_ms: total_ms,
        duration_display: format_duration_ms(total_ms),
        identity: spans
            .iter()
            .find_map(|s| s.identity_label.clone())
            .unwrap_or_else(|| "unknown".to_owned()),
        span_count: spans.len(),
        deny_count: count(SpanStatus::Deny),
        error_count: count(SpanStatus::Error),
        rejected_count: count(SpanStatus::Rejected),
        request_count: spans
            .iter()
            .filter(|s| s.cost_microdollars.is_some())
            .count(),
        cost_display: format_cost(spans.iter().filter_map(|s| s.cost_microdollars).sum()),
    }
}

fn build_span_views(spans: &[Span]) -> Vec<SpanView> {
    let start = spans.iter().map(|s| s.started_at).min();
    spans
        .iter()
        .map(|s| {
            let offset_ms = start.map_or(0, |a| (s.started_at - a).num_milliseconds().max(0));
            SpanView {
                id: s.id.clone(),
                kind: s.kind.as_str(),
                name: s.name.clone(),
                started_at_local: local_time(s.started_at),
                duration_display: format_duration_ms(s.duration_ms),
                status: s.status.as_str(),
                offset_label: format!("+{}", format_duration_ms(offset_ms)),
                provider: dash(s.provider.as_deref()),
                model: dash(s.model.as_deref()),
                cost_display: s
                    .cost_microdollars
                    .map_or_else(|| "—".to_owned(), format_cost),
                latency_display: s
                    .latency_ms
                    .map_or_else(|| "—".to_owned(), format_duration_ms),
            }
        })
        .collect()
}

// Why: a governance or tool span has no provider and no model; an em dash says
// "not applicable here", where an empty cell reads as missing data.
fn dash(value: Option<&str>) -> String {
    value
        .filter(|v| !v.is_empty())
        .map_or_else(|| "—".to_owned(), str::to_owned)
}
