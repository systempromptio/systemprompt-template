//! What the roster renders: its rows, the KPI tiles, the filter chips, the
//! sortable headers and the select menus.
//!
//! Everything here reads a [`RosterQuery`] and writes a view type; nothing
//! here filters — which rows survive is decided in [`super::roster`].

use chrono::Utc;

use crate::handlers::ssr::format::format_token_total;
use crate::handlers::ssr::list_view::{PageWindow, Pagination, SelectOptionView};
use crate::handlers::ssr::types::{FilterChipView, SortHeaderView};

use super::context::{RosterKpiView, RosterRowView};
use super::roster::RosterQuery;

pub(super) fn build_kpis(
    rows: &[RosterRowView],
    departments: &[String],
    query: &RosterQuery,
) -> RosterKpiView {
    let total = i64::try_from(rows.len()).unwrap_or(i64::MAX);
    let active = i64::try_from(rows.iter().filter(|r| r.is_active).count()).unwrap_or(0);
    let no_role = i64::try_from(rows.iter().filter(|r| !r.has_roles).count()).unwrap_or(0);
    let filter = query.field("filter");
    RosterKpiView {
        total,
        active,
        inactive: total - active,
        no_role,
        departments: i64::try_from(departments.len()).unwrap_or(0),
        model_tokens_display: format_token_total(rows.iter().map(|r| r.model_tokens_raw).sum()),
        all_url: query.url_with(&[("filter", ""), ("page", "")]),
        inactive_url: query.url_with(&[("filter", "inactive")]),
        no_role_url: query.url_with(&[("filter", "no-role")]),
        filter_active: filter.is_none(),
        inactive_active: filter == Some("inactive"),
        no_role_active: filter == Some("no-role"),
    }
}

pub(super) fn build_chips(rows: &[RosterRowView], query: &RosterQuery) -> Vec<FilterChipView> {
    let filter = query.field("filter");
    let count = |pred: fn(&RosterRowView) -> bool| {
        i64::try_from(rows.iter().filter(|r| pred(r)).count()).unwrap_or(0)
    };
    [
        ("All", "", count(|_| true)),
        ("Active", "active", count(|r| r.is_active)),
        ("Inactive", "inactive", count(|r| !r.is_active)),
        ("No role", "no-role", count(|r| !r.has_roles)),
    ]
    .into_iter()
    .map(|(label, value, count)| FilterChipView {
        label,
        href: query.url_with(&[("filter", value)]),
        is_active: filter.unwrap_or_default() == value,
        count,
    })
    .collect()
}

fn options(entries: &[String], all_label: &str, selected: Option<&str>) -> Vec<SelectOptionView> {
    let mut out = vec![SelectOptionView {
        value: String::new(),
        label: all_label.to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(entries.iter().map(|value| SelectOptionView {
        value: value.clone(),
        label: value.clone(),
        selected: selected == Some(value.as_str()),
    }));
    out
}

pub(super) fn role_options(roles: &[String], query: &RosterQuery) -> Vec<SelectOptionView> {
    options(roles, "All roles", query.field("role"))
}

pub(super) fn department_options(
    departments: &[String],
    query: &RosterQuery,
) -> Vec<SelectOptionView> {
    options(departments, "All departments", query.field("department"))
}

// Why: the column set, its labels, its alignment class and the one-line
// explanation each header carries. One list so the header row and the sort
// ladder in `roster.rs` cannot drift apart.
const COLUMNS: [(&str, &str, &str, &str); 8] = [
    (
        "name",
        "User",
        "",
        "Display name, falling back to the account id",
    ),
    ("email", "Email", "", "The address on the account"),
    (
        "department",
        "Department",
        "",
        "The department whose access rules this account inherits",
    ),
    (
        "tokens",
        "Tokens",
        "sp-table__cell--num",
        "Active personal access tokens, coloured by when one was last used",
    ),
    (
        "model_tokens",
        "Model tokens",
        "sp-table__cell--num",
        "Lifetime model tokens consumed through the gateway",
    ),
    (
        "sessions",
        "Sessions",
        "sp-table__cell--num",
        "Recorded work sessions",
    ),
    (
        "seen",
        "Last active",
        "sp-table__cell--date",
        "Latest recorded activity; never means no activity at all",
    ),
    (
        "status",
        "Status",
        "sp-col-status",
        "Whether the account may sign in",
    ),
];

pub(super) fn sort_headers(query: &RosterQuery) -> Vec<SortHeaderView> {
    let current = query.sort_key();
    let descending = query.descending();
    COLUMNS
        .into_iter()
        .map(|(key, label, class, hint)| {
            let active = current == key;
            let next_dir = if active && !descending { "desc" } else { "" };
            SortHeaderView {
                label,
                class,
                hint,
                url: query.url_with(&[("sort", key), ("dir", next_dir)]),
                active,
                aria_sort: match (active, descending) {
                    (true, false) => "ascending",
                    (true, true) => "descending",
                    (false, _) => "none",
                },
                indicator: match (active, descending) {
                    (true, false) => "▲",
                    (true, true) => "▼",
                    (false, _) => "",
                },
            }
        })
        .collect()
}

pub(super) fn pagination(query: &RosterQuery, window: PageWindow) -> Pagination {
    let page = window.index;
    let prev_url = (page > 0).then(|| query.url_with(&[("page", &(page - 1).to_string())]));
    let next_url = (page + 1 < window.total_pages)
        .then(|| query.url_with(&[("page", &(page + 1).to_string())]));
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

pub(super) fn initials(name: &str) -> String {
    name.split(|c: char| c.is_whitespace() || c == '-' || c == '.' || c == '@')
        .filter(|part| !part.is_empty())
        .take(2)
        .filter_map(|part| part.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

// Why: the roster shows "3d ago", not a timestamp, because the column exists
// to answer "is this seat being used" at a glance. The exact time stays on
// the cell's title attribute.
pub(super) fn relative(epoch_secs: i64) -> String {
    let delta = Utc::now().timestamp() - epoch_secs;
    match delta {
        d if d < 60 => "just now".to_owned(),
        d if d < 3_600 => format!("{}m ago", d / 60),
        d if d < 86_400 => format!("{}h ago", d / 3_600),
        d if d < 2_592_000 => format!("{}d ago", d / 86_400),
        d => format!("{}mo ago", d / 2_592_000),
    }
}
