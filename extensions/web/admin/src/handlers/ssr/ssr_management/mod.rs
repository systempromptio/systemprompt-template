//! Departments and Access tokens SSR pages.
//!
//! Three admin-only page handlers: the department roster, a single department
//! detail (members + token/cost rollup + top tools), and the access-token
//! console. View-model assembly lives in the `departments` / `access_tokens`
//! children; every filter on the two listings narrows rows in memory, so a
//! view is one URL an operator can send to someone else.

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlError, AdminHtmlResult};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::departments::DEFAULT_DEPARTMENT;
use crate::types::{MarketplaceContext, UserContext};

use super::ssr_helpers::render_typed_page;

mod access_tokens;
mod departments;
mod departments_sort;

use access_tokens::{AccessTokensQuery, ManagementAccessTokensPageData};
use departments::{DepartmentDetailPageData, DepartmentsPageData, DepartmentsQuery};

const DEPARTMENTS_URL: &str = "/admin/departments";
const ACCESS_TOKENS_URL: &str = "/admin/access-tokens";

fn forbidden() -> AdminHtmlError {
    AdminError::Forbidden("Admin access required.".to_owned()).into()
}

fn crumbs(current: impl Into<String>, parent: Option<(&str, &str)>) -> Vec<BreadcrumbView> {
    let mut out = vec![
        BreadcrumbView::link("Admin", "/admin"),
        BreadcrumbView::link("People & access", "/admin/users"),
    ];
    if let Some((label, href)) = parent {
        out.push(BreadcrumbView::link(label, href));
    }
    out.push(BreadcrumbView::current(current));
    out
}

pub(crate) async fn management_departments_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<DepartmentsQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(forbidden());
    }

    let all = repositories::departments::list_departments(&pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "departments: listing failed"))
        .unwrap_or_default();
    let rows = departments::rows(&all, &query);

    let data = DepartmentsPageData {
        page: "departments",
        title: "Departments",
        breadcrumbs: crumbs("Departments", None),
        kpis: departments::kpis(&all),
        sort_headers: departments_sort::sort_headers(&query),
        search: query.search().unwrap_or_default().to_owned(),
        filters_applied: query.search().is_some(),
        clear_url: DEPARTMENTS_URL,
        total: all.len(),
        has_rows: !rows.is_empty(),
        rows,
        can_write: user_ctx.is_admin,
    };

    Ok(render_typed_page(
        &engine,
        "management-departments",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}

pub(crate) async fn management_department_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(forbidden());
    }

    let Some(department) = repositories::departments::find_department(&pool, &id).await? else {
        return Err(AdminError::NotFound("Department not found".to_owned()).into());
    };

    let members = repositories::departments::list_department_members(&pool, &department.name)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "department detail: member listing failed"))
        .unwrap_or_default();
    let top_tools =
        repositories::departments::list_department_top_tools(&pool, &department.name, 10)
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "department detail: top tools failed"))
            .unwrap_or_default();

    let rows = departments::member_rows(&members);
    let data = DepartmentDetailPageData {
        page: "department-detail",
        title: department.name.clone(),
        breadcrumbs: crumbs(
            department.name.clone(),
            Some(("Departments", DEPARTMENTS_URL)),
        ),
        kpis: departments::detail_kpis(&members),
        matrix_url: format!(
            "/admin/access-control?department={}",
            urlencoding::encode(&department.name)
        ),
        users_url: format!(
            "/admin/users?department={}",
            urlencoding::encode(&department.name)
        ),
        is_default: department.name == DEFAULT_DEPARTMENT,
        has_tools: !top_tools.is_empty(),
        top_tools,
        member_count: rows.len(),
        has_members: !rows.is_empty(),
        members: rows,
        department,
        can_write: user_ctx.is_admin,
    };

    Ok(render_typed_page(
        &engine,
        "management-department-detail",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}

pub(crate) async fn management_access_tokens_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AccessTokensQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(forbidden());
    }

    let rows = access_tokens::load_access_tokens(&pool).await;
    let all = access_tokens::build_token_rows(rows);
    let counts = access_tokens::counts(&all);
    let departments = access_tokens::department_names(&all);
    let matching = access_tokens::filtered(&all, &query);
    let user_options = access_tokens::load_token_user_options(&pool).await;

    let data = ManagementAccessTokensPageData {
        page: "access-tokens",
        title: "Access tokens",
        breadcrumbs: crumbs("Access tokens", None),
        total: counts.total,
        active: counts.active,
        expiring_soon: counts.expiring_soon,
        revoked: counts.revoked,
        status_options: access_tokens::status_options(&query),
        department_options: access_tokens::department_options(&departments, &query),
        search: query.search().unwrap_or_default().to_owned(),
        filters_applied: query.any_applied(),
        clear_url: ACCESS_TOKENS_URL,
        shown: matching.len(),
        has_rows: !matching.is_empty(),
        tokens: matching,
        user_options,
        can_write: user_ctx.is_admin,
    };
    Ok(render_typed_page(
        &engine,
        "management-access-tokens",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}
