//! The history listings as the export surface reads them: the same scope,
//! search and side-call toggle the page applies, through the page's own query
//! type.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::AdminError;
use crate::repositories::analytics::conversations::{
    HistoryFilter, HistoryItem, list_history_items,
};
use crate::types::UserContext;

use super::{HistoryQuery, HistoryView, scope_user_ids};

// Why: the export is the listing the reader sees — their own scope, the same
// search — through the page's own query type.
// What an export asks of the history listing, as opposed to who is asking.
pub(crate) struct ExportRequest {
    pub query: HistoryQuery,
    pub view: HistoryView,
    pub limit: i64,
}

pub(crate) async fn export_rows(
    pool: &PgPool,
    user_ctx: &UserContext,
    request: ExportRequest,
) -> Result<(Vec<HistoryItem>, i64), AdminError> {
    let ExportRequest { query, view, limit } = request;
    let scope = view.scope(user_ctx);
    let scope_ids = scope_user_ids(&scope, &query, "export")?;
    Ok(list_history_items(
        pool,
        HistoryFilter {
            scope_user_ids: scope_ids.as_deref(),
            search: query.q.as_deref(),
            include_side_calls: query.show_side(),
        },
        limit,
        0,
    )
    .await?)
}

// Why: the export opens on the listing the reader is looking at — the same
// search, user and side calls.
pub(super) fn export_view(query: &HistoryQuery, view: HistoryView) -> crate::export::ExportView {
    crate::export::ExportView::single(
        view.dataset(),
        &crate::export::view::query_string(&[
            ("q", query.q.as_deref()),
            ("user_id", query.user_id.as_ref().map(UserId::as_str)),
            ("side", query.side.as_deref()),
        ]),
    )
}
