//! Typed template context for the conversation-history page.

use serde::Serialize;
use systemprompt::identifiers::{SessionId, UserId};

use crate::handlers::ssr::list_view::Pagination;
use crate::handlers::ssr::types::BreadcrumbView;

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
    pub show_side: bool,
    pub side_toggle_url: String,
    pub pagination: Pagination,
    pub breadcrumbs: Vec<BreadcrumbView>,
}

#[derive(Debug, Serialize)]
pub(super) struct HistoryRowView {
    pub source: &'static str,
    pub is_gateway: bool,
    pub session_id: Option<SessionId>,
    pub short_id: String,
    // Why: not `title`. The page is rendered inside `{{#> layout title=title}}`,
    // and that hash parameter shadows a field of the same name at every depth,
    // so `{{this.title}}` in the row loop printed the page's own title on every
    // row instead of the conversation's.
    pub conversation_title: String,
    pub user_id: UserId,
    pub is_own: bool,
    pub model: Option<String>,
    pub when_relative: String,
    pub when_at: String,
    pub entries_counted: i64,
    pub side_call_count: i64,
    pub tokens_display: String,
    pub tokens_title: String,
    pub cost_display: Option<String>,
    pub snippet: Option<String>,
    // Why: the gateway detail page is owner-facing, so every viewer gets a
    // link; the transcript detail page is the admin session view, so only an
    // admin does.
    pub detail_url: Option<String>,
}
