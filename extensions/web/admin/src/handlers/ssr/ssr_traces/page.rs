use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::handlers::ssr::ACCESS_DENIED_HTML;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::queries::{
    fetch_session_summary, fetch_trace_ai_calls, fetch_trace_ai_messages,
    fetch_trace_ai_tool_calls, fetch_trace_entities, fetch_trace_events, fetch_trace_governance,
    SessionSummaryRow,
};
use super::views::build_trace_data;
use crate::repositories::perf_grp::traces::resolve_trace_session;

#[derive(Debug, Deserialize)]
pub struct TracesQuery {
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
}

pub async fn traces_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<TracesQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let requested_id = query
        .trace_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            query
                .session_id
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
        })
        .unwrap_or("");

    if requested_id.is_empty() {
        let data = json!({
            "page": "traces",
            "title": "Trace Detail",
            "has_session": false,
            "session_id": "",
        });
        return crate::handlers::ssr::render_page(&engine, "traces", &data, &user_ctx, &mkt_ctx);
    }

    let session_id = match resolve_trace_session(&pool, requested_id).await {
        Ok(Some(sid)) => sid,
        Ok(None) => requested_id.to_string(),
        Err(e) => {
            tracing::warn!(error = %e, id = requested_id, "resolve_trace_session failed");
            requested_id.to_string()
        }
    };
    let session_id = session_id.as_str();

    let (
        events_result,
        governance_result,
        entities_result,
        summary_result,
        ai_calls_result,
        ai_messages_result,
        ai_tool_calls_result,
    ) = tokio::join!(
        fetch_trace_events(&pool, session_id),
        fetch_trace_governance(&pool, session_id),
        fetch_trace_entities(&pool, session_id),
        fetch_session_summary(&pool, session_id),
        fetch_trace_ai_calls(&pool, session_id),
        fetch_trace_ai_messages(&pool, session_id),
        fetch_trace_ai_tool_calls(&pool, session_id),
    );

    let events = unwrap_or_warn(events_result, "trace events");
    let governance = unwrap_or_warn(governance_result, "trace governance");
    let entities = unwrap_or_warn(entities_result, "trace entities");
    let ai_calls = unwrap_or_warn(ai_calls_result, "ai calls");
    let ai_messages = unwrap_or_warn(ai_messages_result, "ai messages");
    let ai_tool_calls = unwrap_or_warn(ai_tool_calls_result, "ai tool calls");
    let summary = summary_result.unwrap_or_else(|e| {
        tracing::error!(error = %e, "Failed to fetch session summary");
        SessionSummaryRow {
            total_events: 0,
            tool_uses: 0,
            prompts: 0,
            errors: 0,
        }
    });

    let data = build_trace_data(
        session_id,
        &events,
        &governance,
        &entities,
        &summary,
        &ai_calls,
        &ai_messages,
        &ai_tool_calls,
    );
    crate::handlers::ssr::render_page(&engine, "traces", &data, &user_ctx, &mkt_ctx)
}

fn unwrap_or_warn<T>(result: Result<Vec<T>, sqlx::Error>, label: &str) -> Vec<T> {
    result.unwrap_or_else(|e| {
        tracing::error!(error = %e, label = label, "Failed to fetch");
        vec![]
    })
}
