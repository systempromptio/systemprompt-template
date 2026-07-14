use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use sqlx::PgPool;

use crate::repositories;
use crate::types::UserContext;
use crate::types::departments::DepartmentInput;

fn forbidden() -> Response {
    (StatusCode::FORBIDDEN, "Admin access required").into_response()
}

pub(crate) async fn list_departments_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }
    match repositories::list_departments(&pool).await {
        Ok(departments) => Json(departments).into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to list departments");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        },
    }
}

pub(crate) async fn create_department_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(input): Json<DepartmentInput>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }
    let trimmed = input.name.trim();
    if trimmed.is_empty() {
        return (StatusCode::BAD_REQUEST, "name must not be empty").into_response();
    }
    if trimmed.eq_ignore_ascii_case("unassigned") {
        return (
            StatusCode::BAD_REQUEST,
            "\"Unassigned\" is reserved for users without a department",
        )
            .into_response();
    }
    let normalized = DepartmentInput {
        name: trimmed.to_owned(),
        description: input.description,
    };
    match repositories::create_department(&pool, &normalized).await {
        Ok(dept) => (StatusCode::CREATED, Json(dept)).into_response(),
        Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
            (StatusCode::CONFLICT, "department name already exists").into_response()
        },
        Err(e) => {
            tracing::warn!(error = %e, "Failed to create department");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        },
    }
}

pub(crate) async fn update_department_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
    Json(input): Json<DepartmentInput>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }
    let trimmed = input.name.trim();
    if trimmed.is_empty() {
        return (StatusCode::BAD_REQUEST, "name must not be empty").into_response();
    }
    if trimmed.eq_ignore_ascii_case("unassigned") {
        return (
            StatusCode::BAD_REQUEST,
            "\"Unassigned\" is reserved for users without a department",
        )
            .into_response();
    }
    let normalized = DepartmentInput {
        name: trimmed.to_owned(),
        description: input.description,
    };
    match repositories::update_department(&pool, &id, &normalized).await {
        Ok(dept) => Json(dept).into_response(),
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
            (StatusCode::CONFLICT, "department name already exists").into_response()
        },
        Err(e) => {
            tracing::warn!(error = %e, "Failed to update department");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        },
    }
}

pub(crate) async fn delete_department_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }
    match repositories::delete_department(&pool, &id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to delete department");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        },
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct AssignDepartmentRequest {
    pub department_name: String,
}

pub(crate) async fn assign_user_to_department_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<String>,
    Json(body): Json<AssignDepartmentRequest>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }
    let dept_name = body.department_name.trim();
    if !dept_name.is_empty()
        && repositories::get_department_by_name(&pool, dept_name)
            .await
            .inspect_err(|e| tracing::warn!(error = %e, dept_name, "get_department_by_name failed"))
            .ok()
            .flatten()
            .is_none()
    {
        return (StatusCode::BAD_REQUEST, "unknown department").into_response();
    }
    match repositories::assign_user_to_department(&pool, &user_id, dept_name).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to assign user to department");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        },
    }
}
