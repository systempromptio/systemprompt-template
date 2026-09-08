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

use crate::util::time_range::{TimeRange, TimeRangePreset, TimeRangeQuery};

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

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SelectOptionView {
    pub value: String,
    pub label: String,
    pub selected: bool,
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

// Why: Drops empty values so a rebuilt URL never carries `&user_id=`.
pub(crate) fn empty_to_none(v: Option<&str>) -> Option<&str> {
    v.filter(|s| !s.is_empty())
}

// Why: The preset name to echo back into the URL.
//
// An explicit `?preset=` wins, then an explicit `from`+`to` pair means
// `custom`; otherwise the parsed range's own preset is authoritative.
pub(crate) fn preset_str(query: &TimeRangeQuery, range: TimeRange) -> String {
    if let Some(p) = empty_to_none(query.preset.as_deref()) {
        return p.to_owned();
    }
    if query.from.is_some() && query.to.is_some() {
        return "custom".to_owned();
    }
    match range.preset {
        TimeRangePreset::Min15 => "15m",
        TimeRangePreset::Hour1 => "1h",
        TimeRangePreset::Hours24 => "24h",
        TimeRangePreset::Days7 => "7d",
        TimeRangePreset::Days30 => "30d",
        TimeRangePreset::Custom => "custom",
    }
    .to_owned()
}

pub(crate) fn time_range_context(
    base_url: &'static str,
    range: TimeRange,
    preset: &str,
) -> TimeRangeContext {
    TimeRangeContext {
        preset: preset.to_owned(),
        from: range.from.to_rfc3339(),
        to: range.to.to_rfc3339(),
        base_url,
        query: "",
        rejected: range.rejected_bounds,
    }
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
        .map(|g| (g.id, g.name))
        .collect();
    let projects = project_rows
        .unwrap_or_default()
        .into_iter()
        .map(|p| (p.id, p.name))
        .collect();
    scope_filter_from_names(user_ctx, scope, base_url, hidden, &groups, &projects)
}

#[expect(
    clippy::too_many_arguments,
    reason = "one view built from the scope request plus the three name sources it renders"
)]
pub(crate) fn scope_filter_from_names(
    user_ctx: &UserContext,
    scope: &ScopeRequest,
    base_url: &str,
    hidden: Vec<(String, String)>,
    groups: &BTreeMap<String, String>,
    projects: &BTreeMap<String, String>,
) -> ScopeFilterView {
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


use crate::repositories::scope::ScopeRequest;
use crate::repositories::{groups, projects};
use crate::types::UserContext;
use sqlx::PgPool;
use std::collections::BTreeMap;
