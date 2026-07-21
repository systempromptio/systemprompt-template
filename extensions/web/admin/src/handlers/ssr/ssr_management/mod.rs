//! `/admin/management/*` — Departments and Devices SSR pages.
//!
//! Three admin-only page handlers: the department roster, a single department
//! detail (members + token/cost rollup + top tools), and the enrolled-device
//! fleet. View-model assembly lives in the `departments` / `devices` children.

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use sqlx::PgPool;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;
use super::ssr_helpers::render_typed_page;

mod departments;
mod devices;

use departments::{DepartmentDetailPageData, DepartmentsPageData, sum_member_totals, url_escape};
use devices::{
    ManagementDevicesPageData, build_device_rows, compute_owner_rowspans, load_device_user_options,
    load_devices,
};

fn forbidden() -> Response {
    (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response()
}

pub(crate) async fn management_departments_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }

    let departments = repositories::departments::list_departments(&pool)
        .await
        .unwrap_or_default();

    let data = DepartmentsPageData {
        page: "management-departments",
        title: "Departments",
        departments,
    };

    render_typed_page(
        &engine,
        "management-departments",
        &data,
        &user_ctx,
        &mkt_ctx,
    )
}

pub(crate) async fn management_devices_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }

    let rows = load_devices(&pool).await;

    let (mut devices, online) = build_device_rows(rows);
    let total = devices.len();
    compute_owner_rowspans(&mut devices);

    let user_options = load_device_user_options(&pool).await;

    let department_options: Vec<String> = repositories::departments::list_departments(&pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|d| d.name)
        .collect();

    let data = ManagementDevicesPageData {
        page: "devices",
        title: "Devices",
        devices,
        total,
        online,
        user_options,
        department_options,
    };
    render_typed_page(&engine, "management-devices", &data, &user_ctx, &mkt_ctx)
}

pub(crate) async fn management_department_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }

    let Ok(Some(department)) = repositories::departments::get_department(&pool, &id).await else {
        return (StatusCode::NOT_FOUND, Html("<h1>Department not found</h1>")).into_response();
    };

    let members = repositories::departments::list_department_members(&pool, &department.name)
        .await
        .unwrap_or_default();
    let member_count = members.len() as i64;

    let top_tools =
        repositories::departments::list_department_top_tools(&pool, &department.name, 10)
            .await
            .unwrap_or_default();

    let totals = sum_member_totals(&members);

    let assignments_url = format!(
        "/admin/access/matrix?department={}",
        url_escape(&department.name)
    );

    let title = format!("Department · {}", department.name);
    let data = DepartmentDetailPageData {
        page: "management-department-detail",
        title,
        department,
        members,
        member_count,
        assignments_url,
        top_tools,
        total_input_tokens: totals.input_tokens,
        total_output_tokens: totals.output_tokens,
        total_requests: totals.requests,
        total_cost_microdollars: totals.cost_microdollars,
    };

    render_typed_page(
        &engine,
        "management-department-detail",
        &data,
        &user_ctx,
        &mkt_ctx,
    )
}
