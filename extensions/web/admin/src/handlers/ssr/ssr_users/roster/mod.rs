//! `/admin/users` — the roster.
//!
//! One flat, paged table rather than the group-bucketed list this page used to
//! be: grouping hid the number an admin comes here for, which is who is
//! spending what. Group membership is a column and a filter instead.
//!
//! Every facet lives in the URL —
//! `?filter=&role=&q=&group=&project=&sort=&dir=&page=` — so a view is
//! bookmarkable, the browser back button works, and the `?filter=unassigned`
//! link the retired Unassigned page redirects to is just another chip.

mod columns;
mod context;
mod view;

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::list_view::PageWindow;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories;
use crate::repositories::scope::ScopeRequest;
use crate::repositories::users::roster::{
    DEFAULT_PAGE_SIZE, RosterFilter, RosterQuery, RosterSort,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use context::{RoleOptionView, RosterContext};

use super::BASE_URL;

// Why: one value so every link builder asks the same question — "the current
// URL, minus these parameters" — instead of each one re-deriving the url it
// must preserve.
pub(super) struct RosterUrlState {
    pub filter: RosterFilter,
    pub role: Option<String>,
    pub search: Option<String>,
    pub group: Option<String>,
    pub project: Option<String>,
    pub sort: RosterSort,
    pub page: i64,
}

impl RosterUrlState {
    // Why: `?a=b&` — always trailing, so a caller appends `key=value` without
    // knowing whether it is the first parameter.
    fn link_prefix(&self, drop: &[&str]) -> String {
        let mut parts: Vec<(String, String)> = Vec::new();
        let mut push = |key: &str, value: String| {
            if !value.is_empty() && !drop.contains(&key) {
                parts.push((key.to_owned(), value));
            }
        };
        push("filter", self.filter.as_str().to_owned());
        push("role", self.role.clone().unwrap_or_default());
        push("q", self.search.clone().unwrap_or_default());
        push("group", self.group.clone().unwrap_or_default());
        push("project", self.project.clone().unwrap_or_default());
        push("sort", self.sort.column.to_owned());
        push(
            "dir",
            if self.sort.descending { "desc" } else { "asc" }.to_owned(),
        );
        if self.page > 0 && !drop.contains(&"page") {
            parts.push(("page".to_owned(), self.page.to_string()));
        }
        let qs = parts
            .into_iter()
            .map(|(k, v)| format!("{k}={}", urlencoding::encode(&v)))
            .collect::<Vec<_>>()
            .join("&");
        if qs.is_empty() {
            format!("{BASE_URL}?")
        } else {
            format!("{BASE_URL}?{qs}&")
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct RosterQueryParams {
    pub filter: Option<String>,
    pub role: Option<String>,
    pub q: Option<String>,
    pub group: Option<String>,
    pub project: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
}

#[expect(
    clippy::too_many_lines,
    reason = "one page assembly per handler; splitting is tracked in docs/tech-debt.md"
)]
pub(crate) async fn users_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<RosterQueryParams>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let url = RosterUrlState {
        filter: RosterFilter::parse_filter(params.filter.as_deref()),
        role: non_empty(params.role.as_deref()),
        search: non_empty(params.q.as_deref()),
        group: non_empty(params.group.as_deref()),
        project: non_empty(params.project.as_deref()),
        sort: RosterSort::parse_sort(params.sort.as_deref(), params.dir.as_deref()),
        page: params.page.unwrap_or(0).max(0),
    };

    let request = ScopeRequest::from_query(&user_ctx, url.group.as_deref(), url.project.as_deref());
    let scope = repositories::scope::membership::get_subject_scope(&pool, &request).await?;

    let query = RosterQuery {
        filter: url.filter,
        role: url.role.clone(),
        search: url.search.clone(),
        sort: url.sort,
        limit: DEFAULT_PAGE_SIZE,
        offset: url.page.saturating_mul(DEFAULT_PAGE_SIZE),
    };

    // Why: the page, the headline numbers, the role facet and the two name maps
    // are independent reads; one round of latency rather than five.
    let (page_res, stats_res, roles_res, names) = tokio::join!(
        repositories::users::roster::list_users_paged(&pool, &scope, &query),
        repositories::users::roster::get_roster_stats(&pool, &scope),
        repositories::users::queries::list_distinct_roles(&pool),
        load_scope_names(&pool),
    );

    let (page_rows, total) =
        page_res.inspect_err(|e| tracing::warn!(error = %e, "roster page query failed"))?;
    let totals = stats_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "roster totals query failed");
        repositories::users::roster::RosterStats::default()
    });
    let roles = roles_res.unwrap_or_default();
    let (group_names, project_names) = names;

    #[expect(
        clippy::cast_possible_wrap,
        reason = "a rendered row count; a page holds fifty rows"
    )]
    let shown = page_rows.len() as i64;
    let window = PageWindow::new(url.page, DEFAULT_PAGE_SIZE, total, shown, "users");

    let rows = view::build_rows(&view::RowInput {
        rows: &page_rows,
        group_names: &group_names,
        project_names: &project_names,
    });

    let data = RosterContext {
        page: "users",
        title: "Users",
        breadcrumbs: vec![BreadcrumbView::current("Users")],
        kpis: view::build_kpis(&url, &totals),
        chips: view::build_chips(&url, &totals),
        role_options: role_options(&roles, url.role.as_deref()),
        search: url.search.clone().unwrap_or_default(),
        scope_filter: super::super::list_view::scope_filter_from_names(
            &user_ctx,
            &request,
            BASE_URL,
            preserved_hidden(&url),
            &group_names,
            &project_names,
        ),
        sort_headers: columns::build_sort_headers(&url, url.sort),
        has_rows: !rows.is_empty(),
        row_count: rows.len(),
        rows,
        pagination: view::build_pagination(&url, window),
        can_write: user_ctx.is_admin,
        role_choices: roles,
    };

    Ok(super::super::render_typed_page(
        &engine, "users", &data, &user_ctx, &mkt_ctx,
    ))
}

fn non_empty(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

// Why: the scope-filter form posts as a GET, so everything the reader had
// selected outside the two scope selects has to ride along as a hidden field
// or submitting the form would silently clear it.
fn preserved_hidden(url: &RosterUrlState) -> Vec<(String, String)> {
    vec![
        ("filter".to_owned(), url.filter.as_str().to_owned()),
        ("role".to_owned(), url.role.clone().unwrap_or_default()),
        ("q".to_owned(), url.search.clone().unwrap_or_default()),
        ("sort".to_owned(), url.sort.column.to_owned()),
        (
            "dir".to_owned(),
            if url.sort.descending { "desc" } else { "asc" }.to_owned(),
        ),
    ]
}


fn role_options(roles: &[String], selected: Option<&str>) -> Vec<RoleOptionView> {
    let mut out = vec![RoleOptionView {
        value: String::new(),
        label: "All roles".to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(roles.iter().map(|role| RoleOptionView {
        selected: selected == Some(role.as_str()),
        value: role.clone(),
        label: role.clone(),
    }));
    out
}

// Why: chips carry names, the URL carries ids. Both maps travel with the page
// rather than being denormalised onto every row. A failed read degrades to
// showing the id, which is still a working link.
async fn load_scope_names(pool: &PgPool) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    let (groups, projects) = tokio::join!(
        repositories::groups::crud::list_groups(pool),
        repositories::projects::crud::list_projects(pool),
    );
    (
        groups
            .unwrap_or_default()
            .into_iter()
            .map(|g| (g.id, g.name))
            .collect(),
        projects
            .unwrap_or_default()
            .into_iter()
            .map(|p| (p.id, p.name))
            .collect(),
    )
}
