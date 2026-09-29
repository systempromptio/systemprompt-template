//! `/api/history/search` — the JSON twin of the history listing: the
//! viewer's own conversations, the same search, window and pagination.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Query, State};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

use crate::error::AdminResult;
use crate::repositories::analytics::conversations::redact_text;
use crate::types::UserContext;

use super::view::detail_url;
use super::{HistoryQuery, HistoryView, PAGE_SIZE, fetch_history_slice};

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

pub(crate) async fn history_search(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<HistoryQuery>,
) -> AdminResult<Response> {
    let slice = fetch_history_slice(&pool, &user_ctx, &query, HistoryView::Own).await?;
    let items = slice
        .items
        .into_iter()
        .map(|item| HistorySearchItem {
            source: item.source.label(),
            detail_url: detail_url(&item, &user_ctx, HistoryView::Own),
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
