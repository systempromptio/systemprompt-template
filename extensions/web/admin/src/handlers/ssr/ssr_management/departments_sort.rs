//! Sortable columns on the department roster.
//!
//! The listing is small enough to order in memory, so the sort is a query
//! parameter pair (`sort`, `dir`) the header links carry and this module
//! turns into both the `components/sort-header` view and the row order.

use crate::handlers::ssr::types::SortHeaderView;
use crate::types::departments::DepartmentSummary;

use super::departments::DepartmentsQuery;

// Why: (key, label, column class, hint) per sortable column, in table order.
// The actions column is not here because it has nothing to sort by.
const COLUMNS: [(&str, &str, &str, &str); 7] = [
    ("name", "Department", "", "The org unit's name"),
    ("description", "Description", "", "What the unit is for"),
    (
        "members",
        "Members",
        "sp-table__cell--num",
        "Accounts placed in it",
    ),
    (
        "rules",
        "Rules",
        "sp-table__cell--num",
        "Access rules its members inherit",
    ),
    (
        "requests",
        "Requests",
        "sp-table__cell--num",
        "Gateway requests, last 30 days",
    ),
    (
        "tokens",
        "Tokens",
        "sp-table__cell--num",
        "Model tokens in and out, last 30 days",
    ),
    ("cost", "Cost", "sp-table__cell--num", "Spend, last 30 days"),
];

impl DepartmentsQuery {
    pub(super) fn sort_key(&self) -> &str {
        self.sort
            .as_deref()
            .and_then(|raw| COLUMNS.iter().find(|(key, ..)| *key == raw))
            .map_or("name", |(key, ..)| key)
    }

    pub(super) fn descending(&self) -> bool {
        self.dir.as_deref() == Some("desc")
    }

    // Why: every URL the page emits carries the search that is set and the
    // sort when it is not the default, so a header link never clears the
    // search and the default view stays the bare path.
    fn url_with(&self, sort: &str, dir: &str) -> String {
        let mut parts: Vec<(&str, String)> = Vec::new();
        if let Some(q) = self.search() {
            parts.push(("q", q.to_owned()));
        }
        if sort != "name" || !dir.is_empty() {
            parts.push(("sort", sort.to_owned()));
        }
        if !dir.is_empty() {
            parts.push(("dir", dir.to_owned()));
        }
        if parts.is_empty() {
            return "/admin/departments".to_owned();
        }
        let query: Vec<String> = parts
            .into_iter()
            .map(|(k, v)| format!("{k}={}", urlencoding::encode(&v)))
            .collect();
        format!("/admin/departments?{}", query.join("&"))
    }
}

pub(super) fn sort_headers(query: &DepartmentsQuery) -> Vec<SortHeaderView> {
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
                url: query.url_with(key, next_dir),
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

pub(super) fn sort_rows(rows: &mut [&DepartmentSummary], query: &DepartmentsQuery) {
    let key = query.sort_key();
    rows.sort_by(|a, b| {
        let ord = match key {
            "description" => a
                .description
                .to_lowercase()
                .cmp(&b.description.to_lowercase()),
            "members" => a.member_count.cmp(&b.member_count),
            "rules" => a.assignment_count.cmp(&b.assignment_count),
            "requests" => a.requests.cmp(&b.requests),
            "tokens" => (a.input_tokens + a.output_tokens).cmp(&(b.input_tokens + b.output_tokens)),
            "cost" => a.cost_microdollars.cmp(&b.cost_microdollars),
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        };
        // Why: ties fall back to the name so a sort by an all-equal column is
        // still a stable, readable order rather than whatever the query gave.
        let ord = ord.then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        if query.descending() {
            ord.reverse()
        } else {
            ord
        }
    });
}
