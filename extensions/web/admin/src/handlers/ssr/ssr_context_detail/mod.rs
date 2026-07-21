//! `/admin/contexts/{context_id}` — single-context detail page.
//!
//! Renders header, KPIs, the chronological conversation transcript (every
//! user/assistant message + tool call interleaved by request and sequence),
//! and the request rollup. Mirrors `core contexts show` plus message detail.

mod context;
mod data;

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use sqlx::PgPool;
use systemprompt::identifiers::ContextId;

use crate::repositories::analytics::context_detail::{
    fetch_context_header, fetch_context_kpis, fetch_context_messages, fetch_context_requests,
    fetch_context_tool_calls,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use data::{build_detail_data, default_kpis};

use super::ACCESS_DENIED_HTML;

const NOT_FOUND_HTML: &str = "<h1>Context not found</h1>\
<p>No context, AI request, or message rows match that context id.</p>";

pub(crate) async fn context_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(context_id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let context_id = ContextId::new(context_id.trim());
    if context_id.as_str().is_empty() {
        return (StatusCode::NOT_FOUND, Html(NOT_FOUND_HTML)).into_response();
    }

    let header = match fetch_context_header(&pool, &context_id).await {
        Ok(Some(h)) => h,
        Ok(None) => return (StatusCode::NOT_FOUND, Html(NOT_FOUND_HTML)).into_response(),
        Err(e) => {
            tracing::error!(error = %e, context_id = %context_id, "fetch_context_header failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, Html(NOT_FOUND_HTML)).into_response();
        },
    };

    let (kpis_res, requests_res, messages_res, tool_calls_res) = tokio::join!(
        fetch_context_kpis(&pool, &context_id),
        fetch_context_requests(&pool, &context_id),
        fetch_context_messages(&pool, &context_id),
        fetch_context_tool_calls(&pool, &context_id),
    );

    let kpis = kpis_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_context_kpis failed");
        default_kpis()
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

    let data = build_detail_data(&header, &kpis, &requests, &messages, &tool_calls);

    super::render_typed_page(&engine, "context-detail", &data, &user_ctx, &mkt_ctx)
}
