//! The roster's sortable column set, and the header row built from it.
//!
//! One list so the header row and the sort contract cannot drift apart: a
//! column with no arm in the repository's `ORDER BY` ladder would render a link
//! that changes nothing.

use crate::repositories::users::roster::RosterSort;

use super::RosterUrlState;
use crate::handlers::ssr::types::SortHeaderView;

// Why: the column set, its labels, its alignment class and the one-line
// explanation each header carries. One list so the header row and the sort
// contract cannot drift apart.
const COLUMNS: [(&str, &str, &str, &str); 8] = [
    (
        "name",
        "User",
        "sp-col-user",
        "Display name, falling back to the account id",
    ),
    (
        "email",
        "Email",
        "sp-col-email",
        "The address the directory vouches for",
    ),
    (
        "roles",
        "Roles",
        "sp-col-roles",
        "Flat, additive role grants on the account",
    ),
    (
        "groups",
        "Belongs to",
        "sp-col-scope",
        "Groups the directory placed this account in, then the projects it is attributed to",
    ),
    (
        "seen",
        "Last seen",
        "sp-table__cell--date",
        "Latest gateway request or console activity; blank means never",
    ),
    (
        "cost",
        "Cost",
        "sp-table__cell--num",
        "Gateway spend over the last 30 days",
    ),
    (
        "requests",
        "Requests",
        "sp-table__cell--num",
        "Gateway requests over the last 30 days",
    ),
    (
        "status",
        "Status",
        "sp-col-status",
        "Whether the account may sign in",
    ),
];

pub(super) fn build_sort_headers(url: &RosterUrlState, sort: RosterSort) -> Vec<SortHeaderView> {
    let prefix = url.link_prefix(&["sort", "dir", "page"]);
    COLUMNS
        .into_iter()
        .map(|(key, label, class, hint)| {
            let active = key == sort.column;
            // Why: an active column toggles; an inactive one opens largest-first
            // (and newest-first for a date), which is what an operator scans for.
            let next_dir = if active && sort.descending {
                "asc"
            } else {
                "desc"
            };
            SortHeaderView {
                label,
                class,
                hint,
                url: format!("{prefix}sort={key}&dir={next_dir}"),
                active,
                aria_sort: aria_sort(active, sort.descending),
                indicator: indicator(active, sort.descending),
            }
        })
        .collect()
}

const fn aria_sort(active: bool, descending: bool) -> &'static str {
    if !active {
        return "none";
    }
    if descending {
        "descending"
    } else {
        "ascending"
    }
}

const fn indicator(active: bool, descending: bool) -> &'static str {
    if !active {
        return "";
    }
    if descending { "▾" } else { "▴" }
}
