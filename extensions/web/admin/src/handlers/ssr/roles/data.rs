//! Which people the roles page shows, and in what order.
//!
//! Every filter narrows the rows in memory rather than in SQL. The read is one
//! pass over the accounts that hold any role — bounded by the number of
//! accounts, not by traffic — and the page needs the unfiltered totals for its
//! role cards anyway, so a second filtered query would only be a second chance
//! for the two to disagree.
//!
//! What the surviving rows are rendered as lives in [`super::rows`]; the seam
//! between the two is [`RolesQuery`], which this module owns and that one only
//! reads.

use serde::Deserialize;

use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::repositories::roles::members::RoleHolderRow;

pub(super) const BASE_URL: &str = "/admin/roles";
pub(super) const PAGE_SIZE: i64 = 50;
// Why: one row per person, so the ceiling is the number of accounts that hold
// any role at all.
pub(super) const MEMBER_CAP: i64 = 2000;
pub(super) const ENTITLEMENT_CAP: i64 = 1000;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RolesQuery {
    pub role: Option<String>,
    pub source: Option<String>,
    pub status: Option<String>,
    pub q: Option<String>,
    pub page: Option<i64>,
    pub sort: Option<String>,
    pub dir: Option<String>,
}

impl RolesQuery {
    pub(super) fn field(&self, name: &str) -> Option<&str> {
        let value = match name {
            "role" => self.role.as_deref(),
            "source" => self.source.as_deref(),
            "status" => self.status.as_deref(),
            "q" => self.q.as_deref(),
            _ => None,
        };
        value.filter(|v| !v.is_empty())
    }

    pub(super) fn any_applied(&self) -> bool {
        ["role", "source", "status", "q"]
            .iter()
            .any(|f| self.field(f).is_some())
    }

    pub(super) fn sort_key(&self) -> &str {
        match self.sort.as_deref() {
            Some(key @ ("person" | "roles" | "requests" | "cost")) => key,
            _ => "person",
        }
    }

    pub(super) fn descending(&self) -> bool {
        self.dir.as_deref() == Some("desc")
    }

    // Why: every URL the page emits carries the filters that are set and
    // nothing else, so a link never resurrects a filter the operator cleared.
    pub(super) fn url_with(&self, overrides: &[(&str, &str)]) -> String {
        let mut parts: Vec<(String, String)> = Vec::new();
        for name in ["role", "source", "status", "q", "sort", "dir"] {
            let overridden = overrides.iter().find(|(k, _)| *k == name);
            let value = match overridden {
                Some((_, v)) => (*v).to_owned(),
                None => match name {
                    "sort" => self.sort_key().to_owned(),
                    "dir" => {
                        if self.descending() {
                            "desc".to_owned()
                        } else {
                            String::new()
                        }
                    },
                    other => self.field(other).unwrap_or_default().to_owned(),
                },
            };
            if !value.is_empty() && (name != "sort" || value != "person") {
                parts.push((name.to_owned(), value));
            }
        }
        for (key, value) in overrides {
            if *key == "page" && !value.is_empty() {
                parts.push(((*key).to_owned(), (*value).to_owned()));
            }
        }
        if parts.is_empty() {
            return BASE_URL.to_owned();
        }
        let query: Vec<String> = parts
            .iter()
            .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
            .collect();
        format!("{BASE_URL}?{}", query.join("&"))
    }
}

// Why: a person holds a role from the directory when the role is in their set
// but not in the manual subset; the two filters ask about the person, not
// about one grant, so "from directory" means at least one such role.
fn holds_directory_role(row: &RoleHolderRow) -> bool {
    row.roles.iter().any(|r| !row.manual_roles.contains(r))
}

fn matches(row: &RoleHolderRow, query: &RolesQuery) -> bool {
    if let Some(role) = query.field("role")
        && !row.roles.iter().any(|r| r == role)
    {
        return false;
    }
    if let Some(source) = query.field("source") {
        let wanted = if source == "manual" {
            !row.manual_roles.is_empty()
        } else {
            holds_directory_role(row)
        };
        if !wanted {
            return false;
        }
    }
    if let Some(status) = query.field("status") {
        let active = status == "active";
        if row.is_active != active {
            return false;
        }
    }
    if let Some(needle) = query.field("q") {
        let needle = needle.to_lowercase();
        let haystack = format!(
            "{} {} {}",
            row.user_id.as_str(),
            row.display_name.clone().unwrap_or_default(),
            row.email
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default()
        )
        .to_lowercase();
        if !haystack.contains(&needle) {
            return false;
        }
    }
    true
}

fn person_key(row: &RoleHolderRow) -> String {
    row.email
        .as_ref()
        .map(ToString::to_string)
        .or_else(|| row.display_name.clone())
        .unwrap_or_else(|| row.user_id.as_str().to_owned())
        .to_lowercase()
}

pub(super) fn filtered_sorted(rows: &[RoleHolderRow], query: &RolesQuery) -> Vec<RoleHolderRow> {
    let mut out: Vec<RoleHolderRow> = rows.iter().filter(|r| matches(r, query)).cloned().collect();
    match query.sort_key() {
        "roles" => out.sort_by_key(|r| (r.roles.len(), person_key(r))),
        "requests" => out.sort_by_key(|r| (r.requests_30d, person_key(r))),
        "cost" => out.sort_by_key(|r| (r.cost_30d_microdollars, person_key(r))),
        _ => out.sort_by_key(person_key),
    }
    if query.descending() {
        out.reverse();
    }
    out
}

pub(super) fn search(query: &RolesQuery) -> String {
    query.field("q").unwrap_or_default().to_owned()
}

pub(super) fn page_index(query: &RolesQuery, total: i64) -> i64 {
    let last = (total.max(1) - 1) / PAGE_SIZE;
    query.page.unwrap_or(0).clamp(0, last)
}

pub(super) fn pagination(query: &RolesQuery, window: PageWindow) -> Pagination {
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
