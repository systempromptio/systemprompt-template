//! `/admin/sessions/{session_id}` — single-session detail page.
//!
//! Renders the header, KPI strip, and three linked tables (contexts, traces,
//! requests) for one `session_id`, mirroring `analytics sessions stats` plus
//! `analytics conversations list` with cross-links to the other entity pages.

mod context;

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use sqlx::PgPool;

use crate::repositories::analytics_grp::session_detail::{
    SessionContextRow, SessionHeader, SessionKpis, SessionRequestRow, SessionTraceRow,
    fetch_session_contexts, fetch_session_header, fetch_session_kpis, fetch_session_requests,
    fetch_session_traces,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use context::{
    SessionContextRowView, SessionDetailPageContext, SessionHeaderView, SessionKpisView,
    SessionRequestRowView, SessionTraceRowView,
};

use super::ACCESS_DENIED_HTML;

const NOT_FOUND_HTML: &str = "<h1>Session not found</h1>\
<p>No AI requests, contexts, or transcript rows match that session id.</p>";

pub(crate) async fn session_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(session_id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let session_id = session_id.trim();
    if session_id.is_empty() {
        return (StatusCode::NOT_FOUND, Html(NOT_FOUND_HTML)).into_response();
    }

    let header = match fetch_session_header(&pool, session_id).await {
        Ok(Some(h)) => h,
        Ok(None) => return (StatusCode::NOT_FOUND, Html(NOT_FOUND_HTML)).into_response(),
        Err(e) => {
            tracing::error!(error = %e, session_id, "fetch_session_header failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, Html(NOT_FOUND_HTML)).into_response();
        },
    };

    let (kpis_res, contexts_res, traces_res, requests_res) = tokio::join!(
        fetch_session_kpis(&pool, session_id),
        fetch_session_contexts(&pool, session_id),
        fetch_session_traces(&pool, session_id),
        fetch_session_requests(&pool, session_id),
    );

    let kpis = kpis_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_session_kpis failed");
        SessionKpis {
            request_count: 0,
            context_count: 0,
            trace_count: 0,
            error_count: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cost_microdollars: 0,
        }
    });
    let contexts = contexts_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_session_contexts failed");
        Vec::new()
    });
    let traces = traces_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_session_traces failed");
        Vec::new()
    });
    let requests = requests_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_session_requests failed");
        Vec::new()
    });

    let data = SessionDetailPageContext {
        page: "session-detail",
        title: format!("Session · {}", short_id(&header.session_id)),
        header: header_view(&header),
        kpis: kpis_view(&kpis),
        has_contexts: !contexts.is_empty(),
        contexts: contexts.iter().map(context_view).collect(),
        has_traces: !traces.is_empty(),
        traces: traces.iter().map(trace_view).collect(),
        has_requests: !requests.is_empty(),
        requests: requests.iter().map(request_view).collect(),
        back_url: "/admin/entities/sessions",
    };

    super::render_typed_page(&engine, "session-detail", &data, &user_ctx, &mkt_ctx)
}

fn header_view(h: &SessionHeader) -> SessionHeaderView {
    SessionHeaderView {
        session_id: h.session_id.clone(),
        session_id_short: short_id(&h.session_id),
        user_id: h.user_id.clone(),
        user_url: h
            .user_id
            .as_ref()
            .map(|u| format!("/admin/user?id={}", urlencoding::encode(u))),
        display_name: h.display_name.clone(),
        department: h.department.clone(),
        started_at: h.started_at.map(|t| t.to_rfc3339()),
        started_at_local: h.started_at.map(|t| {
            t.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }),
        last_activity_at: h.last_activity_at.map(|t| t.to_rfc3339()),
        last_activity_at_local: h.last_activity_at.map(|t| {
            t.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }),
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
        context_id_short: short_id(&c.context_id),
        context_url: format!("/admin/contexts/{}", urlencoding::encode(&c.context_id)),
        name: c.name.clone().unwrap_or_else(|| "—".into()),
        request_count: c.request_count,
        last_request_at: c.last_request_at.map(|t| t.to_rfc3339()),
        last_request_at_local: c.last_request_at.map(|t| {
            t.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }),
        model: c.model.clone(),
    }
}

fn trace_view(t: &SessionTraceRow) -> SessionTraceRowView {
    let duration_ms = match (t.started_at, t.ended_at) {
        (Some(s), Some(e)) => Some((e - s).num_milliseconds().max(0)),
        _ => None,
    };
    SessionTraceRowView {
        trace_id: t.trace_id.clone(),
        trace_id_short: short_id(&t.trace_id),
        trace_url: format!("/admin/traces/{}", urlencoding::encode(&t.trace_id)),
        request_count: t.request_count,
        error_count: t.error_count,
        started_at_local: t.started_at.map(|x| {
            x.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }),
        duration_display: duration_ms.map_or_else(|| "—".to_owned(), format_duration_ms),
    }
}

fn request_view(r: &SessionRequestRow) -> SessionRequestRowView {
    SessionRequestRowView {
        id: r.id.clone(),
        id_short: short_id(&r.id),
        request_url: format!("/admin/requests/{}", urlencoding::encode(&r.id)),
        context_id: r.context_id.clone(),
        context_id_short: r.context_id.as_deref().map(short_id),
        context_url: r
            .context_id
            .as_ref()
            .map(|c| format!("/admin/contexts/{}", urlencoding::encode(c))),
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

fn short_id(id: &str) -> String {
    if id.len() > 12 {
        format!("{}…", &id[..12])
    } else {
        id.to_owned()
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

fn format_duration_ms(ms: i64) -> String {
    if ms < 1000 {
        format!("{ms}ms")
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else if ms < 3_600_000 {
        format!("{}m {}s", ms / 60_000, (ms % 60_000) / 1000)
    } else {
        format!("{}h {}m", ms / 3_600_000, (ms % 3_600_000) / 60_000)
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
