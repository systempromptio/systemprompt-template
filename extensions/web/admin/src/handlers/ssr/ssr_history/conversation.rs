//! `/admin/history/conversations/{context_id}` — one conversation, readable by
//! the person who had it.
//!
//! The admin reader at `/admin/contexts/{id}` renders the same transcript, but
//! it is admin-gated and redirects everyone else away, which is the second
//! reason a user with plenty of gateway traffic saw "no conversations". This
//! page answers the same question for the owner, with the gateway's transcript
//! framing stripped and every body run through the credential redactor.
//! Operational identifiers — trace, request, provider request, route match —
//! are deliberately absent; an admin follows the link to the full context view
//! for those.

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::ContextId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::conversation_header::{
    ConversationStatsView, StatusBadgeView, resolve_title, stats_view, status_badge,
    timeline_display,
};
use crate::handlers::ssr::entity_urls::context_detail_url;
use crate::handlers::ssr::transcript_view::{
    ConversationView, TranscriptOptions, build_conversation,
};
use crate::repositories::analytics::context_detail::{
    find_context_header, get_context_kpis, list_context_messages, list_context_requests,
    list_context_tool_calls,
};
use crate::repositories::analytics::conversations::history_scope_for;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

#[derive(Debug, Serialize)]
struct ConversationPageContext {
    page: &'static str,
    title: String,
    stats: ConversationStatsView,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_badge: Option<StatusBadgeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeline: Option<String>,
    conversation: ConversationView,
    admin_url: Option<String>,
}

fn not_found() -> AdminError {
    AdminError::NotFound("No conversation matches that id.".to_owned())
}

pub(crate) async fn history_conversation_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(context_id): Path<String>,
) -> AdminHtmlResult<Response> {
    // Why: `ContextId::new` panics on anything that is not a UUID, and this
    // segment comes straight off the URL.
    let Ok(context_id) = ContextId::try_new(context_id.trim()) else {
        return Err(not_found().into());
    };

    let Some(header) = find_context_header(&pool, &context_id).await? else {
        return Err(not_found().into());
    };

    // Why: 404, not 403, and the same message as a missing id. A 403 would
    // confirm that a context with this id exists and say whose it is not,
    // which turns the URL into an oracle for enumerating other people's
    // conversation ids.
    let scope = history_scope_for(&user_ctx);
    let owned = header
        .user_id
        .as_ref()
        .is_some_and(|owner| scope.may_view(owner));
    if !owned {
        return Err(not_found().into());
    }

    let (kpis_res, requests_res, messages_res, tool_calls_res) = tokio::join!(
        get_context_kpis(&pool, &context_id),
        list_context_requests(&pool, &context_id),
        list_context_messages(&pool, &context_id),
        list_context_tool_calls(&pool, &context_id),
    );

    let kpis = kpis_res?;
    let requests = requests_res?;
    let messages = messages_res?;
    // Why: same reasoning as the admin reader — a transcript that silently
    // drops its tool calls reads as a complete conversation in which the
    // agent did nothing, and nothing on the page says otherwise.
    let tool_calls = tool_calls_res?;

    let conversation = build_conversation(
        &messages,
        &tool_calls,
        &requests,
        TranscriptOptions::owner_facing(),
    );

    let data = ConversationPageContext {
        page: "history",
        title: resolve_title(&header, &conversation),
        stats: stats_view(&kpis),
        status_badge: status_badge(header.hook_status.as_deref(), kpis.error_count),
        timeline: timeline_display(kpis.first_request_at, kpis.last_request_at),
        conversation,
        admin_url: user_ctx.is_admin.then(|| context_detail_url(&context_id)),
    };

    Ok(super::super::render_typed_page(
        &engine,
        "history-conversation",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}
