//! `/admin/history` — a user's own conversation history, searchable.
//!
//! The one analytics surface a non-admin may reach: every viewer sees their
//! own conversations, and admin/auditor keep the unrestricted view. Both ways
//! a conversation gets recorded are listed — the Claude Code session
//! transcript written by the stop hook, and the gateway conversation grouped
//! out of `ai_requests` — because a user who only ever calls `/v1/messages`
//! has no transcript rows at all and used to be shown an empty page. The scope
//! is resolved server-side per request; asking for a `user_id` outside it is a
//! 403. Snippets pass through the transcript redactor before rendering, and
//! raw bodies stay on the existing admin/auditor-gated endpoint. Side calls
//! — cache probes and utility calls — are hidden unless `?side=1`.

mod context;
mod conversation;
mod view;

pub(crate) use conversation::history_conversation_page;

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Query, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::list_view::PageWindow;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::analytics::conversations::{
    HistoryItem, HistoryScope, history_scope_for, list_history_items, redact_text,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use context::{HistoryPageContext, HistoryRowView};
use view::{build_pagination, detail_url, row_view, scope_label, side_toggle_url};

const PAGE_SIZE: i64 = 50;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct HistoryQuery {
    q: Option<String>,
    user_id: Option<UserId>,
    page: Option<i64>,
    side: Option<String>,
}

impl HistoryQuery {
    fn show_side(&self) -> bool {
        self.side.as_deref() == Some("1")
    }
}

#[derive(Debug, Serialize)]
struct HistorySearchItem {
    source: &'static str,
    session_id: Option<SessionId>,
    context_id: Option<ContextId>,
    user_id: UserId,
    ai_title: Option<String>,
    preview: Option<String>,
    model: Option<String>,
    started_at: Option<String>,
    captured_at: String,
    entries_counted: i64,
    total_input_tokens: i64,
    total_output_tokens: i64,
    cost_microdollars: i64,
    side_call_count: i64,
    rank: Option<f32>,
    snippet: Option<String>,
    detail_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct HistorySearchEnvelope {
    items: Vec<HistorySearchItem>,
    total: i64,
    page: i64,
    page_size: i64,
}

struct HistorySlice {
    scope: HistoryScope,
    items: Vec<HistoryItem>,
    total: i64,
    page: i64,
}

async fn fetch_history_slice(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: &HistoryQuery,
) -> Result<HistorySlice, AdminError> {
    let scope = history_scope_for(user_ctx);

    let target = query
        .user_id
        .as_ref()
        .filter(|u| !u.as_str().trim().is_empty());
    let scope_ids = match target {
        Some(target_id) => {
            if !scope.may_view(target_id) {
                return Err(AdminError::Forbidden(
                    "You may only view conversation history within your own scope.".to_owned(),
                ));
            }
            Some(vec![target_id.as_str().to_owned()])
        },
        None => scope.user_ids(),
    };

    let page = query.page.unwrap_or(0).max(0);
    let (items, total) = list_history_items(
        pool,
        scope_ids.as_deref(),
        query.q.as_deref(),
        query.show_side(),
        PAGE_SIZE,
        page * PAGE_SIZE,
    )
    .await?;
    Ok(HistorySlice {
        scope,
        items,
        total,
        page,
    })
}

pub(crate) async fn history_search(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<HistoryQuery>,
) -> AdminResult<Response> {
    let slice = fetch_history_slice(&pool, &user_ctx, &query).await?;
    let items = slice
        .items
        .into_iter()
        .map(|item| HistorySearchItem {
            source: item.source.label(),
            detail_url: detail_url(&item, &user_ctx),
            session_id: item.session_id.clone(),
            context_id: item.context_id.clone(),
            user_id: item.user_id,
            ai_title: item.title,
            preview: item.preview.map(|s| redact_text(&s).0),
            model: item.model,
            started_at: item.started_at.map(|t| t.to_rfc3339()),
            captured_at: item.last_at.to_rfc3339(),
            entries_counted: item.turns,
            total_input_tokens: item.total_input_tokens,
            total_output_tokens: item.total_output_tokens,
            cost_microdollars: item.cost_microdollars,
            side_call_count: item.side_call_count,
            rank: item.rank,
            snippet: item.snippet.map(|s| redact_text(&s).0),
        })
        .collect();
    Ok(Json(HistorySearchEnvelope {
        items,
        total: slice.total,
        page: slice.page,
        page_size: PAGE_SIZE,
    })
    .into_response())
}

pub(crate) async fn history_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<HistoryQuery>,
) -> AdminHtmlResult<Response> {
    let slice = fetch_history_slice(&pool, &user_ctx, &query).await?;

    let rows: Vec<HistoryRowView> = slice
        .items
        .iter()
        .map(|item| row_view(item, &user_ctx))
        .collect();

    let window = PageWindow::new(
        slice.page,
        PAGE_SIZE,
        slice.total,
        i64::try_from(rows.len()).unwrap_or(PAGE_SIZE),
        "conversations",
    );
    let data = HistoryPageContext {
        page: "history",
        title: "My Conversations",
        search_query: query.q.clone().unwrap_or_default(),
        filter_user_id: query.user_id.as_ref().map(|u| u.as_str().to_owned()),
        viewer_is_admin: user_ctx.is_admin,
        scope_label: scope_label(&slice.scope),
        has_rows: !rows.is_empty(),
        rows,
        show_side: query.show_side(),
        side_toggle_url: side_toggle_url(&query),
        pagination: build_pagination(&query, window),
        breadcrumbs: vec![
            BreadcrumbView::link("Account", "/admin/profile"),
            BreadcrumbView::current("My conversations"),
        ],
    };
    Ok(super::render_typed_page(
        &engine, "history", &data, &user_ctx, &mkt_ctx,
    ))
}
