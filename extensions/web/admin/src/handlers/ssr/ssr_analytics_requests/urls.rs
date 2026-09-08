//! URL builders for the Inference Requests page.
//!
//! Every link the page emits — tab bar, chip removal, Clear, the breakdown
//! rows' drill-through, and pagination — rebuilds the current query string
//! minus the one parameter it is changing, so the reader never loses the
//! window or the filters they picked.

use urlencoding::encode as urlencode;

use crate::handlers::ssr::list_view::{PageWindow, Pagination};

use crate::handlers::ssr::list_view::Chip;

use super::context::{RequestsTab, TabLinkView};
use super::{BASE_URL, RequestsQuery};

// Why: a breakdown row drills into the Log tab carrying its own dimension as a
// filter, on top of whatever filters and window are already active.
pub(super) fn log_filter_url(query: &RequestsQuery, param: &str, value: &str) -> String {
    let qs = preserved_query_string(query, &[param, "page", "tab"]);
    let mut url = format!("{BASE_URL}?tab=log&{param}={}", urlencode(value));
    if !qs.is_empty() {
        url.push('&');
        url.push_str(&qs);
    }
    url
}

// Why: every tab link carries the current window and the active filters so
// switching tabs never silently resets what the reader chose, but drops the
// page number, which means nothing on another tab.
pub(super) fn tab_links(
    active: RequestsTab,
    query: &RequestsQuery,
    total: i64,
) -> Vec<TabLinkView> {
    const TABS: [(RequestsTab, &str); 5] = [
        (RequestsTab::Log, "Log"),
        (RequestsTab::Overview, "Overview"),
        (RequestsTab::Models, "Models"),
        (RequestsTab::Providers, "Providers"),
        (RequestsTab::Status, "Status"),
    ];

    let qs = preserved_query_string(query, &["tab", "page"]);
    TABS.iter()
        .map(|&(tab, label)| {
            let mut href = format!("{BASE_URL}?tab={}", tab.as_str());
            if !qs.is_empty() {
                href.push('&');
                href.push_str(&qs);
            }
            TabLinkView {
                slug: tab.as_str(),
                label,
                href,
                is_active: tab == active,
                count: (tab == RequestsTab::Log).then_some(total),
            }
        })
        .collect()
}

// Why: removing a chip drops just that parameter and keeps the tab, window,
// and every other filter intact.
pub(super) fn active_chips(query: &RequestsQuery) -> Vec<Chip> {
    let mut chips = Vec::new();
    for (param, group_label, value) in [
        ("model", "Model", query.model.as_deref()),
        ("provider", "Provider", query.provider.as_deref()),
        ("status", "Status", query.status.as_deref()),
        ("tool", "Tool", query.tool.as_deref()),
        ("group", "Group", query.group.as_deref()),
        ("project", "Project", query.project.as_deref()),
        ("q", "Search", query.q.as_deref()),
    ] {
        let Some(value) = value.filter(|s| !s.is_empty()) else {
            continue;
        };
        let qs = preserved_query_string(query, &[param, "page"]);
        chips.push(Chip {
            group_label,
            label: value.to_owned(),
            value: value.to_owned(),
            remove_url: if qs.is_empty() {
                BASE_URL.to_owned()
            } else {
                format!("{BASE_URL}?{qs}")
            },
        });
    }
    chips
}

// Why: Clear drops every filter but keeps the reader on the tab and window
// they are looking at.
pub(super) fn clear_url(query: &RequestsQuery) -> String {
    let qs = preserved_query_string(
        query,
        &[
            "model", "provider", "status", "tool", "q", "user_id", "agent_id", "page",
        ],
    );
    if qs.is_empty() {
        BASE_URL.to_owned()
    } else {
        format!("{BASE_URL}?{qs}")
    }
}

pub(super) fn preserved_query_string(query: &RequestsQuery, drop: &[&str]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let pairs_str: [(&str, Option<&str>); 15] = [
        ("tab", query.tab.as_deref()),
        ("group", query.group.as_deref()),
        ("project", query.project.as_deref()),
        ("preset", query.preset.as_deref()),
        ("from", query.from.as_deref()),
        ("to", query.to.as_deref()),
        (
            "user_id",
            query
                .user_id
                .as_ref()
                .map(systemprompt::identifiers::UserId::as_str),
        ),
        (
            "agent_id",
            query
                .agent_id
                .as_ref()
                .map(systemprompt::identifiers::AgentId::as_str),
        ),
        ("model", query.model.as_deref()),
        ("provider", query.provider.as_deref()),
        ("status", query.status.as_deref()),
        ("tool", query.tool.as_deref()),
        ("q", query.q.as_deref()),
        ("sort", query.sort.as_deref()),
        ("dir", query.dir.as_deref()),
    ];
    for (name, value) in pairs_str {
        if drop.contains(&name) {
            continue;
        }
        let Some(v) = value.filter(|s| !s.is_empty()) else {
            continue;
        };
        parts.push(format!("{}={}", name, urlencode(v)));
    }
    if !drop.contains(&"page")
        && let Some(p) = query.page.filter(|p| *p > 0)
    {
        parts.push(format!("page={p}"));
    }
    parts.join("&")
}

// Why: a sortable header keeps every filter and the window, and drops the page
// — re-sorting a list puts different rows on page one, so staying on page 7
// would land the reader somewhere they did not choose.
pub(super) fn sort_url_prefix(query: &RequestsQuery) -> String {
    let qs = preserved_query_string(query, &["sort", "dir", "page"]);
    if qs.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs}&")
    }
}

// Why: the export is the same question the table is answering, so it carries
// every filter and the window rather than dumping the raw table.
pub(super) fn csv_url(query: &RequestsQuery) -> String {
    let qs = preserved_query_string(query, &["tab", "page"]);
    if qs.is_empty() {
        "/admin/requests.csv".to_owned()
    } else {
        format!("/admin/requests.csv?{qs}")
    }
}

pub(super) fn build_pagination(query: &RequestsQuery, window: PageWindow) -> Pagination {
    let page = window.index;
    let qs = preserved_query_string(query, &["page"]);
    let prefix = if qs.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs}&")
    };
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
