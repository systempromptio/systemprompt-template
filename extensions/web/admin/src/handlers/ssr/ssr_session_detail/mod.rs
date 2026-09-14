//! `/admin/sessions/{session_id}` — single-session detail page.
//!
//! Renders the header, KPI strip, and three linked tables (contexts, traces,
//! requests) for one `session_id`, mirroring `analytics sessions stats` plus
//! `analytics conversations list` with cross-links to the other entity pages.

mod context;
mod quality;

use crate::error::AdminError;
use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

use crate::error::AdminHtmlResult;
use crate::handlers::ssr::entity_urls::{context_detail_url, request_detail_url, trace_detail_url};
use crate::handlers::ssr::format::{format_cost, format_duration_ms, local_time, short_id};
use crate::repositories::analytics::session_detail::{
    SessionContextRow, SessionHeader, SessionKpis, SessionRequestRow, SessionTraceRow,
    find_session_header, get_session_kpis, list_session_contexts, list_session_requests,
    list_session_traces,
};
use crate::repositories::analytics::session_quality::{
    SessionAnalysisSummary, SessionRatingRow, find_session_analysis, list_session_ratings,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use context::{
    BreadcrumbView, SessionContextRowView, SessionDetailPageContext, SessionHeaderView,
    SessionKpisView, SessionRequestRowView, SessionTraceRowView,
};


struct SessionRead {
    kpis: SessionKpis,
    contexts: Vec<SessionContextRow>,
    traces: Vec<SessionTraceRow>,
    requests: Vec<SessionRequestRow>,
    analysis: Option<SessionAnalysisSummary>,
    ratings: Vec<SessionRatingRow>,
}

// Why: every section is best-effort once the header has resolved — a panel
// that cannot load renders empty rather than taking the page down.
async fn load_session(pool: &PgPool, session_id: &SessionId) -> SessionRead {
    let (kpis, contexts, traces, requests, analysis, ratings) = tokio::join!(
        get_session_kpis(pool, session_id),
        list_session_contexts(pool, session_id),
        list_session_traces(pool, session_id),
        list_session_requests(pool, session_id),
        find_session_analysis(pool, session_id),
        list_session_ratings(pool, session_id),
    );
    SessionRead {
        kpis: kpis
            .inspect_err(|e| tracing::warn!(error = %e, "get_session_kpis failed"))
            .unwrap_or_default(),
        contexts: contexts
            .inspect_err(|e| tracing::warn!(error = %e, "list_session_contexts failed"))
            .unwrap_or_default(),
        traces: traces
            .inspect_err(|e| tracing::warn!(error = %e, "list_session_traces failed"))
            .unwrap_or_default(),
        requests: requests
            .inspect_err(|e| tracing::warn!(error = %e, "list_session_requests failed"))
            .unwrap_or_default(),
        analysis: analysis
            .inspect_err(|e| tracing::warn!(error = %e, "find_session_analysis failed"))
            .unwrap_or_default(),
        ratings: ratings
            .inspect_err(|e| tracing::warn!(error = %e, "list_session_ratings failed"))
            .unwrap_or_default(),
    }
}

pub(crate) async fn session_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(session_id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let session_id = SessionId::new(session_id.trim());
    let header = if session_id.as_str().is_empty() {
        None
    } else {
        find_session_header(&pool, &session_id).await?
    };
    let Some(header) = header else {
        return Err(AdminError::NotFound(
            "No AI requests, contexts, or transcript rows match that session id.".to_owned(),
        )
        .into());
    };

    let read = load_session(&pool, &session_id).await;
    let rating_average = quality::rating_average(&read.ratings);
    let rating_views: Vec<_> = read.ratings.iter().map(quality::rating_view).collect();

    let data = SessionDetailPageContext {
        page: "session-detail",
        title: format!("Session · {}", short_id(header.session_id.as_str())),
        breadcrumbs: breadcrumbs(&header),
        header: header_view(&header),
        kpis: kpis_view(&read.kpis),
        has_contexts: !read.contexts.is_empty(),
        contexts: read.contexts.iter().map(context_view).collect(),
        has_traces: !read.traces.is_empty(),
        traces: read.traces.iter().map(trace_view).collect(),
        has_requests: !read.requests.is_empty(),
        requests: read.requests.iter().map(request_view).collect(),
        back_url: "/admin/sessions",
        analysis: read.analysis.as_ref().map(quality::analysis_view),
        has_ratings: !rating_views.is_empty(),
        rating_count: rating_views.len(),
        rating_average,
        ratings: rating_views,
    };

    Ok(super::render_typed_page(
        &engine,
        "session-detail",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn breadcrumbs(h: &SessionHeader) -> Vec<BreadcrumbView> {
    vec![
        BreadcrumbView::link("Sessions", "/admin/sessions"),
        BreadcrumbView::current(short_id(h.session_id.as_str())),
    ]
}

fn header_view(h: &SessionHeader) -> SessionHeaderView {
    SessionHeaderView {
        session_id: h.session_id.clone(),
        session_id_short: short_id(h.session_id.as_str()),
        user_id: h.user_id.clone(),
        user_url: h
            .user_id
            .as_ref()
            .map(|u| format!("/admin/users/{}", urlencoding::encode(u.as_str()))),
        display_name: h.display_name.clone(),
        groups_display: (!h.groups.is_empty()).then(|| h.groups.join(", ")),
        started_at: h.started_at.map(|t| t.to_rfc3339()),
        started_at_local: h.started_at.map(local_time),
        last_activity_at: h.last_activity_at.map(|t| t.to_rfc3339()),
        last_activity_at_local: h.last_activity_at.map(local_time),
        duration_display: duration_display(h.started_at, h.last_activity_at),
        status: h.status.as_deref().unwrap_or("ended").to_owned(),
        model: h.model.clone(),
        plugin_id: h.plugin_id.clone(),
        ai_title: h.ai_title.clone(),
    }
}

fn kpis_view(k: &SessionKpis) -> SessionKpisView {
    SessionKpisView {
        request_count: k.request_count,
        context_count: k.context_count,
        trace_count: k.trace_count,
        error_count: k.error_count,
        total_input_tokens: k.total_input_tokens,
        total_output_tokens: k.total_output_tokens,
        total_tokens: k.total_input_tokens + k.total_output_tokens,
        total_cost_microdollars: k.total_cost_microdollars,
        total_cost_display: format_cost(k.total_cost_microdollars),
    }
}

fn context_view(c: &SessionContextRow) -> SessionContextRowView {
    SessionContextRowView {
        context_id: c.context_id.clone(),
        context_id_short: short_id(c.context_id.as_str()),
        context_url: context_detail_url(&c.context_id),
        name: c.name.clone().unwrap_or_else(|| "—".into()),
        request_count: c.request_count,
        last_request_at: c.last_request_at.map(|t| t.to_rfc3339()),
        last_request_at_local: c.last_request_at.map(local_time),
        model: c.model.clone(),
        total_tokens: c.total_input_tokens + c.total_output_tokens,
        token_display: format!(
            "{} in / {} out",
            c.total_input_tokens, c.total_output_tokens
        ),
        cost_display: format_cost(c.cost_microdollars),
        error_count: c.error_count,
    }
}

fn trace_view(t: &SessionTraceRow) -> SessionTraceRowView {
    let duration_ms = match (t.started_at, t.ended_at) {
        (Some(s), Some(e)) => Some((e - s).num_milliseconds().max(0)),
        _ => None,
    };
    SessionTraceRowView {
        trace_id: t.trace_id.clone(),
        trace_id_short: short_id(t.trace_id.as_str()),
        trace_url: trace_detail_url(&t.trace_id),
        request_count: t.request_count,
        error_count: t.error_count,
        started_at_local: t.started_at.map(local_time),
        duration_display: duration_ms.map_or_else(|| "—".to_owned(), format_duration_ms),
    }
}

fn request_view(r: &SessionRequestRow) -> SessionRequestRowView {
    SessionRequestRowView {
        id: r.id.clone(),
        id_short: short_id(r.id.as_str()),
        request_url: request_detail_url(&r.id),
        context_id: r.context_id.clone(),
        context_id_short: r.context_id.as_ref().map(|c| short_id(c.as_str())),
        context_url: r.context_id.as_ref().map(context_detail_url),
        trace_id: r.trace_id.clone(),
        trace_id_short: r.trace_id.as_ref().map(|t| short_id(t.as_str())),
        trace_url: r.trace_id.as_ref().map(trace_detail_url),
        model: r.model.clone().unwrap_or_else(|| "—".to_owned()),
        status: r.status.clone(),
        is_error: r.status == "failed",
        // Why: a rejected request never reached a provider, so it has no model
        // to name. Showing it as an error would report a policy refusal as a
        // fault to chase upstream.
        is_rejected: r.status == "rejected",
        latency_display: r
            .latency_ms
            .map_or_else(|| "—".to_owned(), |ms| format!("{ms}ms")),
        cost_display: format_cost(r.cost_microdollars),
        created_at_local: local_time(r.created_at),
    }
}

fn duration_display(
    start: Option<chrono::DateTime<chrono::Utc>>,
    end: Option<chrono::DateTime<chrono::Utc>>,
) -> String {
    match (start, end) {
        (Some(s), Some(e)) => format_duration_ms((e - s).num_milliseconds().max(0)),
        _ => "—".to_owned(),
    }
}
