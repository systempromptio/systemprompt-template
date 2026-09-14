//! Row shaping and link building for the history page: the title a row
//! shows, the detail page a viewer may open, and the query state every page
//! and toggle link carries.

use systemprompt::identifiers::UserId;

use crate::handlers::ssr::entity_urls::{context_detail_url, session_detail_url};
use crate::handlers::ssr::format::{format_cost, format_token_total, relative_time};
use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::handlers::ssr::transcript_view::short_id;
use crate::repositories::analytics::conversations::{
    HistoryItem, HistoryScope, HistorySource, redact_text,
};
use crate::types::UserContext;

use super::context::HistoryRowView;
use super::{HistoryQuery, HistoryView};

pub(super) fn scope_label(scope: &HistoryScope) -> String {
    match scope {
        HistoryScope::All => "all users".to_owned(),
        HistoryScope::Users(_) => "your own conversations".to_owned(),
    }
}

// Why: the org-wide listing at `/admin/conversations` is this page with a
// wider scope, so every query link takes the base url of whichever of the two
// is rendering rather than assuming `/admin/history`.

// Why: a gateway conversation has an owner-facing detail page, so every viewer
// gets the link. A transcript's only detail page is the admin session view.
//
// The org-wide listing sends its rows to the admin context view instead: it is
// already admin-gated, renders the same transcript unredacted, and carries the
// operational identifiers an admin came to that page for.
pub(crate) fn detail_url(
    item: &HistoryItem,
    viewer: &UserContext,
    view: HistoryView,
) -> Option<String> {
    match item.source {
        HistorySource::Gateway => item.context_id.as_ref().map(|c| match view {
            HistoryView::Org => context_detail_url(c),
            HistoryView::Own => format!(
                "/admin/history/conversations/{}",
                urlencoding::encode(c.as_str())
            ),
        }),
        HistorySource::Transcript => viewer
            .is_admin
            .then(|| item.session_id.as_ref().map(session_detail_url))
            .flatten(),
    }
}

fn row_identity(item: &HistoryItem) -> String {
    item.session_id
        .as_ref()
        .map(|s| s.as_str().to_owned())
        .or_else(|| item.context_id.as_ref().map(|c| c.as_str().to_owned()))
        .unwrap_or_default()
}

// Why: a slash command's opening prompt is the client's XML envelope, not
// anything a person typed — whole screens of `<command-message>` wrappers that
// tell a reader nothing and are identical across rows. Name such a row by the
// command it ran instead.
pub fn command_name(prompt: &str) -> Option<String> {
    let open = "<command-name>";
    let close = "</command-name>";
    let start = prompt.find(open)? + open.len();
    let end = prompt[start..].find(close)? + start;
    let name = prompt[start..end].trim().trim_start_matches('/');
    (!name.is_empty()).then(|| format!("/{name}"))
}

// Why: a gateway conversation without a recorded title is named by its
// opening prompt, so a row never shows a bare id where words exist.
fn conversation_title(item: &HistoryItem, is_gateway: bool, short: &str) -> String {
    item.title
        .clone()
        .filter(|t| !t.trim().is_empty())
        .or_else(|| {
            item.preview
                .as_deref()
                .map(|p| redact_text(p).0)
                .filter(|p| !p.trim().is_empty())
                .map(|p| command_name(&p).unwrap_or_else(|| p.chars().take(160).collect()))
        })
        .unwrap_or_else(|| {
            if is_gateway {
                format!("Conversation {short}")
            } else {
                format!("Session {short}")
            }
        })
}

pub(crate) fn row_view(
    item: &HistoryItem,
    viewer: &UserContext,
    view: HistoryView,
) -> HistoryRowView {
    let when = item.last_at;
    let identity = row_identity(item);
    let short = short_id(&identity);
    let is_gateway = item.source == HistorySource::Gateway;
    HistoryRowView {
        source: item.source.label(),
        is_gateway,
        session_id: item.session_id.clone(),
        conversation_title: conversation_title(item, is_gateway, &short),
        short_id: short,
        user_id: item.user_id.clone(),
        user_label: item.user_label.clone(),
        is_own: item.user_id == viewer.user_id,
        model: item.model.clone(),
        when_relative: relative_time(when),
        when_at: when.to_rfc3339(),
        entries_counted: item.turns,
        side_call_count: item.side_call_count,
        tokens_display: format_token_total(item.total_input_tokens + item.total_output_tokens),
        tokens_title: format!(
            "{} in / {} out",
            item.total_input_tokens, item.total_output_tokens
        ),
        cost_display: is_gateway.then(|| format_cost(item.cost_microdollars)),
        snippet: item.snippet.as_deref().map(|s| redact_text(s).0),
        detail_url: detail_url(item, viewer, view),
    }
}

fn query_parts(query: &HistoryQuery, keep_side: bool) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    if let Some(q) = query.q.as_deref().filter(|s| !s.is_empty()) {
        parts.push(format!("q={}", urlencoding::encode(q)));
    }
    if let Some(u) = query
        .user_id
        .as_ref()
        .map(UserId::as_str)
        .filter(|s| !s.is_empty())
    {
        parts.push(format!("user_id={}", urlencoding::encode(u)));
    }
    if keep_side && query.show_side() {
        parts.push("side=1".to_owned());
    }
    parts
}

pub(super) fn side_toggle_url(query: &HistoryQuery, base: &str) -> String {
    let mut parts = query_parts(query, false);
    if !query.show_side() {
        parts.push("side=1".to_owned());
    }
    if parts.is_empty() {
        base.to_owned()
    } else {
        format!("{base}?{}", parts.join("&"))
    }
}

pub(super) fn build_pagination(query: &HistoryQuery, window: PageWindow, base: &str) -> Pagination {
    let parts = query_parts(query, true);
    let prefix = if parts.is_empty() {
        format!("{base}?")
    } else {
        format!("{base}?{}&", parts.join("&"))
    };
    let page = window.index;
    let prev_url = (page > 0).then(|| format!("{prefix}page={}", page - 1));
    let next_url = (page + 1 < window.total_pages).then(|| format!("{prefix}page={}", page + 1));
    let (first_row, last_row) = window.bounds();
    Pagination {
        current_page: page + 1,
        total_pages: window.total_pages,
        first_row,
        last_row,
        total_rows: window.total_rows,
        noun: window.noun,
        has_prev: prev_url.is_some(),
        has_next: next_url.is_some(),
        prev_url,
        next_url,
    }
}
