//! `/admin/contexts/{context_id}` — single-context detail page.
//!
//! Renders header, KPIs, the chronological conversation transcript (every
//! user/assistant message + tool call interleaved by request and sequence),
//! and the request rollup. Mirrors `core contexts show` plus message detail.

mod context;
mod data;

use crate::error::AdminError;
use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use sqlx::PgPool;
use systemprompt::identifiers::ContextId;

use crate::error::AdminHtmlResult;
use crate::repositories::analytics::context_detail::{
    find_context_header, get_context_kpis, list_context_messages, list_context_requests,
    list_context_tool_calls,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use data::{build_detail_data, default_kpis};


pub(crate) async fn context_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(context_id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let context_id = ContextId::new(context_id.trim());
    if context_id.as_str().is_empty() {
        return Err(AdminError::NotFound(
            "No context, AI request, or message rows match that context id.".to_owned(),
        )
        .into());
    }

    let Some(header) = find_context_header(&pool, &context_id).await? else {
        return Err(AdminError::NotFound(
            "No context, AI request, or message rows match that context id.".to_owned(),
        )
        .into());
    };

    let (kpis_res, requests_res, messages_res, tool_calls_res) = tokio::join!(
        get_context_kpis(&pool, &context_id),
        list_context_requests(&pool, &context_id),
        list_context_messages(&pool, &context_id),
        list_context_tool_calls(&pool, &context_id),
    );

    let kpis = kpis_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "get_context_kpis failed");
        default_kpis()
    });
    let requests = requests_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_context_requests failed");
        Vec::new()
    });
    let messages = messages_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_context_messages failed");
        Vec::new()
    });
    let tool_calls = tool_calls_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_context_tool_calls failed");
        Vec::new()
    });

    let data = build_detail_data(&header, &kpis, &requests, &messages, &tool_calls);

    Ok(super::render_typed_page(
        &engine,
        "context-detail",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}
