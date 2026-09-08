//! Link building and row shaping for the conversations list: the query state
//! every sort link, page link and tab carries, the sortable headers, the
//! pagination footer, and the repository-row → template-row transforms.

use std::collections::HashMap;

use systemprompt::identifiers::UserId;
use urlencoding::encode as urlencode;

use crate::handlers::ssr::entity_urls::context_detail_url;
use crate::handlers::ssr::format::{format_cost, format_token_total, relative_time, short_id};
use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::handlers::ssr::types::{SortHeaderView, TabLinkView};
use crate::repositories::analytics::conversation_rows::{
    ConversationRow, ConversationSort, UserConversationSummary,
};

use super::context::{ContextsSortHeaders, ConversationItemView, UserSummaryView};
use super::{BASE_URL, ContextsListQuery, PAGE_SIZE};

pub(super) fn preserved_query_string(query: &ContextsListQuery, drop: &[&str]) -> String {
    let pairs: [(&str, Option<&str>); 11] = [
        ("group", query.group.as_deref()),
        ("project", query.project.as_deref()),
        ("user_id", query.user_id.as_ref().map(UserId::as_str)),
        ("model", query.model.as_deref()),
        ("q", query.q.as_deref()),
        ("since", query.since.as_deref()),
        ("preset", query.preset.as_deref()),
        ("view", query.view.as_deref()),
        ("side", query.side.as_deref()),
        ("sort", query.sort.as_deref()),
        ("dir", query.dir.as_deref()),
    ];
    pairs
        .iter()
        .filter(|(name, _)| !drop.contains(name))
        .filter_map(|(name, val)| {
            val.filter(|s| !s.is_empty())
                .map(|v| format!("{}={}", name, urlencode(v)))
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn link_prefix(query: &ContextsListQuery, drop: &[&str]) -> String {
    let qs = preserved_query_string(query, drop);
    if qs.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs}&")
    }
}

pub(super) fn side_toggle_url(query: &ContextsListQuery, active: bool) -> String {
    let qs = preserved_query_string(query, &["side", "page"]);
    match (qs.is_empty(), active) {
        (true, true) => BASE_URL.to_owned(),
        (true, false) => format!("{BASE_URL}?side=1"),
        (false, true) => format!("{BASE_URL}?{qs}"),
        (false, false) => format!("{BASE_URL}?{qs}&side=1"),
    }
}

pub(super) fn build_pagination(query: &ContextsListQuery, window: PageWindow) -> Pagination {
    let page = window.index;
    let prefix = link_prefix(query, &["page"]);
    let prev_url = (page > 0).then(|| format!("{prefix}page={}", page.saturating_sub(1)));
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

pub(super) fn build_sort_headers(
    query: &ContextsListQuery,
    active_col: ConversationSort,
    is_desc: bool,
) -> ContextsSortHeaders {
    let prefix = link_prefix(query, &["sort", "dir", "page"]);
    let header = |col: ConversationSort, label: &'static str, class: &'static str, hint| {
        let active = col == active_col;
        let next_dir = if active && is_desc { "asc" } else { "desc" };
        SortHeaderView {
            label,
            class,
            hint,
            url: format!("{prefix}sort={}&dir={next_dir}", col.as_str()),
            active,
            aria_sort: if active {
                if is_desc { "descending" } else { "ascending" }
            } else {
                "none"
            },
            indicator: if active {
                if is_desc { "▼" } else { "▲" }
            } else {
                "↕"
            },
        }
    };
    ContextsSortHeaders {
        activity: header(
            ConversationSort::Activity,
            "Last activity",
            "sp-col-date",
            "Most recent request on the conversation",
        ),
        turns: header(
            ConversationSort::Turns,
            "Turns",
            "sp-table__cell--num",
            "Human prompts answered in this conversation",
        ),
        tokens: header(
            ConversationSort::Tokens,
            "Tokens",
            "sp-table__cell--num",
            "Input plus output tokens across the conversation",
        ),
        cost: header(
            ConversationSort::Cost,
            "Cost",
            "sp-table__cell--num",
            "Billed cost across the conversation, side calls included",
        ),
    }
}

pub(super) fn page_offset(page: i64) -> i64 {
    page.max(0) * PAGE_SIZE
}

// Why: links, not buttons, so a view is bookmarkable and survives a reload
// without JavaScript.
pub(super) fn view_tabs(query: &ContextsListQuery, active: &str) -> Vec<TabLinkView> {
    let prefix = link_prefix(query, &["view", "page"]);
    [("users", "By user"), ("all", "All")]
        .into_iter()
        .map(|(slug, label)| TabLinkView {
            slug,
            label,
            href: format!("{prefix}view={slug}"),
            is_active: slug == active,
            count: None,
        })
        .collect()
}

fn user_url(user_id: &UserId) -> String {
    format!("/admin/users/{}", urlencode(user_id.as_str()))
}

pub(super) fn conversation_item(c: &ConversationRow) -> ConversationItemView {
    let user_label = c
        .display_name
        .clone()
        .or_else(|| c.user_id.as_ref().map(|u| short_id(u.as_str())))
        .unwrap_or_else(|| "—".to_owned());
    ConversationItemView {
        context_id: c.context_id.clone(),
        detail_url: context_detail_url(&c.context_id),
        conversation_title: c.title.clone(),
        user_id: c.user_id.clone(),
        user_label,
        user_url: c.user_id.as_ref().map(user_url),
        model: c.model.clone(),
        turn_count: c.turn_count,
        side_call_count: c.side_call_count,
        tool_call_count: c.tool_call_count,
        error_count: c.error_count,
        tokens_display: format_token_total(c.total_input_tokens + c.total_output_tokens),
        tokens_title: format!(
            "{} in / {} out",
            c.total_input_tokens, c.total_output_tokens
        ),
        cost_display: format_cost(c.total_cost_microdollars),
        last_at: c.last_at.map(|t| t.to_rfc3339()),
        last_relative: c.last_at.map(relative_time),
    }
}

pub(super) fn group_by_user(
    rows: &[ConversationRow],
) -> HashMap<String, Vec<ConversationItemView>> {
    let mut out: HashMap<String, Vec<ConversationItemView>> = HashMap::new();
    for c in rows {
        let Some(user) = c.user_id.as_ref() else {
            continue;
        };
        out.entry(user.as_str().to_owned())
            .or_default()
            .push(conversation_item(c));
    }
    out
}

pub(super) fn user_summary(
    s: &UserConversationSummary,
    by_user: &HashMap<String, Vec<ConversationItemView>>,
    params: &ContextsListQuery,
) -> UserSummaryView {
    let conversations = by_user.get(s.user_id.as_str()).cloned().unwrap_or_default();
    UserSummaryView {
        user_id: s.user_id.clone(),
        user_label: s
            .display_name
            .clone()
            .unwrap_or_else(|| s.user_id.as_str().to_owned()),
        user_url: user_url(&s.user_id),
        conversation_count: s.conversation_count,
        tokens_display: format_token_total(s.total_tokens),
        all_conversations_url: format!(
            "{}view=all&user_id={}",
            link_prefix(params, &["view", "user_id", "page"]),
            urlencode(s.user_id.as_str())
        ),
        turn_count: s.turn_count,
        side_call_count: s.side_call_count,
        cost_display: format_cost(s.total_cost_microdollars),
        last_at: s.last_at.map(|t| t.to_rfc3339()),
        last_relative: s.last_at.map(relative_time),
        latest: s.latest.as_ref().map(conversation_item),
        models: s.models.clone(),
        has_conversations: !conversations.is_empty(),
        conversations,
    }
}
