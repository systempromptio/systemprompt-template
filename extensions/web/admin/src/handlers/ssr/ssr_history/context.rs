//! Typed template context for the conversation-history page.

use serde::Serialize;
use systemprompt::identifiers::{SessionId, UserId};

use crate::handlers::ssr::list_view::Pagination;

#[derive(Debug, Serialize)]
pub(super) struct HistoryPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub search_query: String,
    pub filter_user_id: Option<String>,
    pub viewer_is_admin: bool,
    pub scope_label: String,
    pub has_rows: bool,
    pub rows: Vec<HistoryRowView>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize)]
pub(super) struct HistoryRowView {
    pub session_id: SessionId,
    pub session_id_short: String,
    pub title: String,
    pub user_id: UserId,
    pub is_own: bool,
    pub model: Option<String>,
    pub when_local: String,
    pub entries_counted: i32,
    pub tokens_display: String,
    pub snippet: Option<String>,
    pub session_url: Option<String>,
}
