//! Which column the projects listing is ordered by, and the header links
//! that change it.
//!
//! The sort key is validated against a fixed list rather than passed through,
//! so a query string can only choose an ordering the page actually offers.

use super::super::types::{ProjectSortHeaderView, ProjectSortHeaders};
use super::BASE_URL;
use crate::repositories::projects::usage::ProjectRollup;

use super::list::ListQuery;
use super::pct;

const SORT_KEYS: [&str; 7] = [
    "name", "members", "groups", "requests", "cost", "tools", "skills",
];

pub(super) fn sort_key(requested: Option<&str>) -> &'static str {
    requested
        .and_then(|r| SORT_KEYS.iter().find(|k| **k == r).copied())
        .unwrap_or("cost")
}

pub(super) fn sort_rows(rows: &mut [ProjectRollup], key: &str, descending: bool) {
    match key {
        "name" => rows.sort_by_key(|a| a.name.to_lowercase()),
        "members" => rows.sort_by_key(|r| r.member_count),
        "groups" => rows.sort_by_key(|r| r.group_count),
        "requests" => rows.sort_by_key(|r| r.requests),
        "tools" => rows.sort_by_key(|r| pct(r.tool_success, r.tool_calls)),
        "skills" => rows.sort_by_key(|r| r.skills_used),
        _ => rows.sort_by_key(|r| r.cost_microdollars),
    }
    if descending {
        rows.reverse();
    }
}

fn preserved(query: &ListQuery) -> String {
    query
        .q
        .as_ref()
        .filter(|q| !q.is_empty())
        .map_or_else(String::new, |q| format!("&q={}", urlencoding::encode(q)))
}

pub(super) fn page_url(query: &ListQuery) -> String {
    let dir = if query.dir.as_deref() == Some("asc") {
        "asc"
    } else {
        "desc"
    };
    format!(
        "{BASE_URL}?sort={}&dir={dir}{}",
        sort_key(query.sort.as_deref()),
        preserved(query)
    )
}

pub(super) fn sort_headers(
    query: &ListQuery,
    active: &str,
    descending: bool,
) -> ProjectSortHeaders {
    let tail = preserved(query);
    let header =
        |key: &'static str, label: &'static str, class: &'static str, hint: &'static str| {
            let is_active = key == active;
            // Why: an active column toggles; an inactive one opens largest-first,
            // which is the order every one of these questions is asked in.
            let next = if is_active && descending {
                "asc"
            } else {
                "desc"
            };
            ProjectSortHeaderView {
                label,
                class,
                hint,
                url: format!("{BASE_URL}?sort={key}&dir={next}{tail}"),
                active: is_active,
                aria_sort: if is_active {
                    if descending {
                        "descending"
                    } else {
                        "ascending"
                    }
                } else {
                    "none"
                },
                indicator: if is_active {
                    if descending { "▼" } else { "▲" }
                } else {
                    "↕"
                },
            }
        };
    ProjectSortHeaders {
        name: header(
            "name",
            "Project",
            "sp-p-projects__col-name",
            "The project id and what it is for",
        ),
        members: header(
            "members",
            "Members",
            "sp-table__cell--num",
            "People holding membership, and how many were active in the window",
        ),
        groups: header(
            "groups",
            "Groups",
            "sp-table__cell--num",
            "Distinct groups the members belong to — who feeds this project",
        ),
        requests: header(
            "requests",
            "Requests",
            "sp-table__cell--num",
            "Gateway requests attributed to this project, exclusively",
        ),
        cost: header(
            "cost",
            "Cost",
            "sp-table__cell--num",
            "Billed cost over the window",
        ),
        tools: header(
            "tools",
            "Tool success",
            "sp-table__cell--num",
            "Share of MCP tool calls that returned success",
        ),
        skills: header(
            "skills",
            "Skills",
            "sp-table__cell--num",
            "Distinct skills the project's people invoked",
        ),
    }
}
