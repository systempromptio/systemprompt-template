//! `/admin/contexts/{context_id}` — the conversation reader.
//!
//! Renders the title bar, stat strip and one of three tabs (`?tab=`): the
//! turn-by-turn conversation, the request rollup with each request's kind,
//! and what the session touched.

mod context;
mod data;

use crate::error::AdminError;
use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::ContextId;

use crate::error::AdminHtmlResult;
use crate::repositories::analytics::context_detail::{
    find_context_header, get_context_kpis, list_context_messages, list_context_requests,
    list_context_tool_calls,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use data::{DetailInputs, build_detail_data, default_kpis, resolve_tab};

#[derive(Debug, Deserialize)]
pub(crate) struct ContextTabQuery {
    tab: Option<String>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "axum handler: every argument is an extractor"
)]
pub(crate) async fn context_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(context_id): Path<String>,
    Query(query): Query<ContextTabQuery>,
) -> AdminHtmlResult<Response> {
    // Why: Raw evidence requires admin/auditor. The write_boundaries contract
    // covers rejection of console readers.
    if !crate::repositories::analytics::conversations::has_full_history_view(&user_ctx) {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    // Why: `ContextId::new` panics on anything that is not a UUID, and this
    // segment comes straight off the URL.
    let Ok(context_id) = ContextId::try_new(context_id.trim()) else {
        return Err(AdminError::NotFound(
            "No conversation, AI request, or message rows match that context id.".to_owned(),
        )
        .into());
    };

    let Some(header) = find_context_header(&pool, &context_id).await? else {
        return Err(AdminError::NotFound(
            "No conversation, AI request, or message rows match that context id.".to_owned(),
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
    // Why: the transcript interleaves messages and tool calls under one empty
    // state, and no KPI counts tool calls — so losing these renders a
    // complete-looking conversation with every tool invocation silently removed
    // and nothing anywhere on the page indicating an omission. On a surface
    // whose purpose is evidencing what an agent did, that is redaction, not
    // degradation.
    let tool_calls = tool_calls_res?;

    // Why: only a context that belongs to a session has entity links; a
    // gateway-only context has no hook events, so the panel is legitimately
    // empty rather than missing.
    let entity_links = match header.session_id.as_ref() {
        Some(session_id) => {
            crate::repositories::dashboard::conversation_analytics::list_session_entity_links(
                &pool, session_id,
            )
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "list_session_entity_links failed");
                Vec::new()
            })
        },
        None => Vec::new(),
    };

    let data = build_detail_data(
        &header,
        &DetailInputs {
            kpis: &kpis,
            requests: &requests,
            messages: &messages,
            tool_calls: &tool_calls,
            entity_links: &entity_links,
            active_tab: resolve_tab(query.tab.as_deref()),
        },
    );

    Ok(super::render_typed_page(
        &engine,
        "context-detail",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}
