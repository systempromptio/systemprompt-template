//! `/admin/projects` and `/admin/projects/{project_id}`.
//!
//! A project is work attribution and an ACL subject; a group is people and
//! entitlement. The listing answers "where is the spend and is the tooling
//! working"; the detail page answers it for one project across three tabs —
//! who is on it, what it did, and how it is configured.

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::list_view::Pagination;
use super::types::BreadcrumbView;

mod detail;
mod detail_view;
mod list;
mod sort;

pub(super) const BASE_URL: &str = "/admin/projects";
pub(super) const PAGE_SIZE: i64 = 50;
pub(super) const WINDOW_LABEL: &str = "Last 30 days · exclusive attribution";

// Why: which tab of the detail page a request asked for.
#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct ProjectTabQuery {
    tab: Option<String>,
}

pub(crate) async fn projects_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<list::ListQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let data = list::page_data(&pool, &query, user_ctx.is_admin).await;
    Ok(super::render_typed_page(
        &engine, "projects", &data, &user_ctx, &mkt_ctx,
    ))
}

#[expect(
    clippy::too_many_arguments,
    reason = "page query plumbing; splitting the parameters is tracked in docs/tech-debt.md"
)]
pub(crate) async fn project_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(project_id): Path<String>,
    Query(query): Query<ProjectTabQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let Some(project) = repositories::projects::crud::find_project(&pool, &project_id).await?
    else {
        return Err(AdminError::NotFound("No such project.".to_owned()).into());
    };

    let tab = active_tab(query.tab.as_deref());
    let loaded = detail::load(&pool, &project_id, &user_ctx).await;
    let usage = if tab == "usage" {
        Some(detail::load_usage(&pool, &project_id).await)
    } else {
        None
    };
    let (mappings, rules) = if tab == "settings" {
        settings_reads(&pool, &project_id).await
    } else {
        (Vec::new(), Vec::new())
    };

    let data = detail::page_data(
        &project,
        &loaded,
        usage.as_ref(),
        tab,
        &user_ctx,
        mappings,
        &rules,
    );
    Ok(super::render_typed_page(
        &engine,
        "project-detail",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn active_tab(requested: Option<&str>) -> &'static str {
    match requested {
        Some("usage") => "usage",
        Some("settings") => "settings",
        _ => "members",
    }
}

// Why: only the rules whose subject is this project are shown, and the page
// says so — a project is an ACL subject, not a container of entitlement, so
// listing everything a member happens to reach would misattribute grants that
// came from their group.
async fn settings_reads(
    pool: &PgPool,
    project_id: &str,
) -> (
    Vec<crate::types::projects::ProjectAdMappingRow>,
    Vec<crate::types::access_control::AccessControlRule>,
) {
    let mappings = repositories::projects::mappings::list_project_ad_mappings(pool, project_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "project AD mappings failed"))
        .unwrap_or_default();
    let rules = repositories::users::access_control::list_all_rules(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access rules failed"))
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r.rule_type.as_str() == "project" && r.rule_value == project_id)
        .collect();
    (mappings, rules)
}

pub(super) fn breadcrumbs(name: &str) -> Vec<BreadcrumbView> {
    vec![
        BreadcrumbView::link("Projects", BASE_URL),
        BreadcrumbView::current(name),
    ]
}

// Why: an integer percentage with the zero-denominator case decided once, so
// no caller invents its own answer for "no calls at all".
pub(super) fn pct(part: i64, whole: i64) -> i64 {
    if whole <= 0 {
        return 0;
    }
    (part * 100 / whole).clamp(0, 100)
}

#[expect(
    clippy::too_many_arguments,
    reason = "page query plumbing; splitting the parameters is tracked in docs/tech-debt.md"
)]
pub(super) fn pagination(
    page: i64,
    total: i64,
    offset: i64,
    shown: i64,
    noun: &'static str,
    base: &str,
) -> Pagination {
    let total_pages = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    Pagination {
        current_page: page.min(total_pages),
        total_pages,
        first_row: if shown == 0 { 0 } else { offset + 1 },
        last_row: offset + shown,
        total_rows: total,
        noun,
        has_prev: page > 1,
        has_next: page < total_pages,
        prev_url: (page > 1).then(|| format!("{base}&page={}", page - 1)),
        next_url: (page < total_pages).then(|| format!("{base}&page={}", page + 1)),
    }
}
