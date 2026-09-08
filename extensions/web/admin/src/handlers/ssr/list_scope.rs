//! Shared group and project filters for scoped listings.

use super::list_view::SelectOptionView;
use serde::Serialize;

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

// Why: the option lists are read here rather than passed in so that every
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
    reason = "combines the caller scope, navigation state, and two membership catalogs"
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
