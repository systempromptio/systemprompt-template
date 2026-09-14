//! The pieces every filtered entity-list page renders the same way.
//!
//! Every filtered list page binds to the same URL contract —
//! `?preset=&from=&to=&<facet>=&page=` — and feeds the same
//! `components/time-range`, `components/identity-filter-ribbon` and pagination
//! partials. The types below are the serialisation those partials read, and
//! the functions build them from a page's own query parameters expressed as a
//! `(name, value)` slice, so a list page supplies its parameter list and
//! inherits the behaviour rather than copying it.

pub(crate) use systemprompt_web_shared::pagination::PageWindow;

use serde::Serialize;
use std::collections::BTreeMap;

use sqlx::PgPool;

use crate::repositories::scope::ScopeRequest;
use crate::repositories::{groups, projects};
use crate::types::UserContext;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SelectOptionView {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HiddenFieldView {
    pub name: String,
    pub value: String,
}

// Why: The `components/scope-filter` partial's context: the two selects an
// admin narrows a listing with, `group` and `project`. Hidden for everyone
// else — a non-console caller's scope is their own groups and nothing on the
// page can widen it.
#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct ScopeFilterView {
    pub show: bool,
    pub base_url: String,
    pub groups: Vec<SelectOptionView>,
    pub projects: Vec<SelectOptionView>,
    pub hidden: Vec<HiddenFieldView>,
}

// Why: Why: the option lists are read here rather than passed in so that every
// list page inherits the same filter by calling one function, and a group
// created in the dashboard appears on all of them at once. A failed read
// degrades to the "all" option alone rather than failing the page.
pub(crate) async fn scope_filter_view(
    pool: &PgPool,
    user_ctx: &UserContext,
    scope: &ScopeRequest,
    base_url: &str,
    hidden: Vec<(String, String)>,
) -> ScopeFilterView {
    if !user_ctx.is_console {
        return ScopeFilterView::default();
    }
    let (group_rows, project_rows) = tokio::join!(
        groups::crud::list_groups(pool),
        projects::crud::list_projects(pool)
    );
    let groups = group_rows
        .unwrap_or_default()
        .into_iter()
        .map(|g| (g.id.as_str().to_owned(), g.name))
        .collect();
    let projects = project_rows
        .unwrap_or_default()
        .into_iter()
        .map(|p| (p.id.as_str().to_owned(), p.name))
        .collect();
    scope_filter_from_names(
        user_ctx,
        scope,
        base_url,
        hidden,
        ScopeNames {
            groups: &groups,
            projects: &projects,
        },
    )
}

// Why: Carries display names for the containers a scope filter can select.
#[derive(Clone, Copy)]
pub(crate) struct ScopeNames<'a> {
    pub(crate) groups: &'a BTreeMap<String, String>,
    pub(crate) projects: &'a BTreeMap<String, String>,
}

pub(crate) fn scope_filter_from_names(
    user_ctx: &UserContext,
    scope: &ScopeRequest,
    base_url: &str,
    hidden: Vec<(String, String)>,
    names: ScopeNames<'_>,
) -> ScopeFilterView {
    let ScopeNames { groups, projects } = names;
    ScopeFilterView {
        show: user_ctx.is_console,
        base_url: base_url.to_owned(),
        groups: options(
            "All groups",
            scope.group.as_deref(),
            groups.iter().map(|(id, name)| (id.clone(), name.clone())),
        ),
        projects: options(
            "All projects",
            scope.project.as_deref(),
            projects.iter().map(|(id, name)| (id.clone(), name.clone())),
        ),
        hidden: hidden
            .into_iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(name, value)| HiddenFieldView { name, value })
            .collect(),
    }
}

fn options(
    all_label: &str,
    selected: Option<&str>,
    rows: impl Iterator<Item = (String, String)>,
) -> Vec<SelectOptionView> {
    let mut out = vec![SelectOptionView {
        value: String::new(),
        label: all_label.to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(rows.map(|(value, label)| SelectOptionView {
        selected: selected == Some(value.as_str()),
        value,
        label,
    }));
    out
}

#[derive(Debug, Serialize)]
pub(crate) struct TimeRangeContext {
    pub(crate) preset: String,
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) base_url: &'static str,
    pub(crate) query: &'static str,
    // Why: carried from `TimeRange::rejected_bounds` so the partial can say
    // the window is the default rather than the one the URL asked for. A
    // listing that answers for a different window without saying so is read as
    // the answer to the question that was asked.
    pub(crate) rejected: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct Preserved {
    pub(crate) name: &'static str,
    pub(crate) value: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AnnotatedOption {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) count: i64,
    pub(crate) selected: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct Chip {
    pub(crate) group_label: &'static str,
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) remove_url: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct Pagination {
    pub(crate) current_page: i64,
    pub(crate) total_pages: i64,
    // Why: 1-based row range for "Showing 1-50 of 54"; `first_row` is 0 only
    // when the page is empty.
    pub(crate) first_row: i64,
    pub(crate) last_row: i64,
    pub(crate) total_rows: i64,
    pub(crate) noun: &'static str,
    pub(crate) has_prev: bool,
    pub(crate) has_next: bool,
    pub(crate) prev_url: Option<String>,
    pub(crate) next_url: Option<String>,
}
