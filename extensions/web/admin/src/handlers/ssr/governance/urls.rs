//! URL assembly for the governance page: tabs, sort headers, pagination.
//!
//! Every control on the page is a link, so a view is bookmarkable and the
//! server renders only what the URL asked for. That makes URL construction the
//! page's single most repeated operation, and it lives here so a control can
//! never drop a parameter another control set.

use serde::Serialize;

use super::{BASE_URL, GovernanceQuery};
use crate::handlers::ssr::list_view::{PageWindow, Pagination};

// Why: One `(name, value)` the page's URLs carry forward.
type Param = (&'static str, String);

// Why: Everything on the query string except the parameter being overridden.
fn params(query: &GovernanceQuery) -> Vec<Param> {
    let mut out: Vec<Param> = Vec::new();
    let mut push = |name: &'static str, value: Option<&String>| {
        if let Some(v) = value.filter(|v| !v.is_empty()) {
            out.push((name, v.clone()));
        }
    };
    push("tab", query.tab.as_ref());
    push("preset", query.preset.as_ref());
    push("from", query.from.as_ref());
    push("to", query.to.as_ref());
    push("group", query.group.as_ref());
    push("project", query.project.as_ref());
    push("policy", query.policy.as_ref());
    push("decision", query.decision.as_ref());
    push("category", query.category.as_ref());
    push("blocked", query.blocked.as_ref());
    push("q", query.q.as_ref());
    push("sort", query.sort.as_ref());
    push("dir", query.dir.as_ref());
    if let Some(page) = query.page.filter(|p| *p > 0) {
        out.push(("page", page.to_string()));
    }
    out
}

// Why: The page URL with `overrides` applied; an empty override drops the key.
pub(super) fn url_with(query: &GovernanceQuery, overrides: &[(&'static str, &str)]) -> String {
    let mut kept: Vec<Param> = params(query)
        .into_iter()
        .filter(|(name, _)| !overrides.iter().any(|(o, _)| o == name))
        .collect();
    for (name, value) in overrides {
        if !value.is_empty() {
            kept.push((name, (*value).to_owned()));
        }
    }
    render(&kept)
}

// Why: The same URL with the paging cursor reset, which every filter change
// wants.
pub(super) fn filter_url(query: &GovernanceQuery, overrides: &[(&'static str, &str)]) -> String {
    let mut all: Vec<(&'static str, &str)> = overrides.to_vec();
    all.push(("page", ""));
    url_with(query, &all)
}

fn render(params: &[Param]) -> String {
    if params.is_empty() {
        return BASE_URL.to_owned();
    }
    let query = params
        .iter()
        .map(|(name, value)| format!("{name}={}", urlencoding::encode(value)))
        .collect::<Vec<_>>()
        .join("&");
    format!("{BASE_URL}?{query}")
}

// Why: One column header, sortable when it carries a `url`.
#[derive(Debug, Serialize)]
pub(super) struct ColumnHeader {
    pub(super) label: &'static str,
    pub(super) class: &'static str,
    pub(super) url: Option<String>,
    pub(super) active: bool,
    pub(super) aria_sort: &'static str,
    pub(super) indicator: &'static str,
    pub(super) hint: Option<&'static str>,
}

impl ColumnHeader {
    pub(super) const fn plain(label: &'static str, class: &'static str) -> Self {
        Self {
            label,
            class,
            url: None,
            active: false,
            aria_sort: "none",
            indicator: "",
            hint: None,
        }
    }

    // Why: clicking the active column flips the direction rather than
    // re-sorting the same way, which is the behaviour every table in the
    // console has and the only one a reader will guess.
    #[expect(
        clippy::too_many_arguments,
        reason = "page query plumbing; splitting the parameters is tracked in docs/tech-debt.md"
    )]
    pub(super) fn sortable(
        label: &'static str,
        class: &'static str,
        key: &'static str,
        query: &GovernanceQuery,
        active_key: &str,
        ascending: bool,
    ) -> Self {
        let active = active_key == key;
        let next = if active && ascending { "desc" } else { "asc" };
        Self {
            label,
            class,
            url: Some(filter_url(query, &[("sort", key), ("dir", next)])),
            active,
            aria_sort: if !active {
                "none"
            } else if ascending {
                "ascending"
            } else {
                "descending"
            },
            indicator: if !active {
                ""
            } else if ascending {
                "\u{2191}"
            } else {
                "\u{2193}"
            },
            hint: None,
        }
    }
}

pub(super) fn build_pagination(
    query: &GovernanceQuery,
    window: PageWindow,
    noun: &'static str,
) -> Pagination {
    let page = window.index;
    let (first_row, last_row) = window.bounds();
    Pagination {
        current_page: page + 1,
        total_pages: window.total_pages,
        first_row,
        last_row,
        total_rows: window.total_rows,
        noun,
        has_prev: page > 0,
        has_next: page + 1 < window.total_pages,
        prev_url: (page > 0).then(|| url_with(query, &[("page", &(page - 1).to_string())])),
        next_url: (page + 1 < window.total_pages)
            .then(|| url_with(query, &[("page", &(page + 1).to_string())])),
    }
}
