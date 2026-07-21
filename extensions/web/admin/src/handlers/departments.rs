//! HTTP handlers for department CRUD and membership.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminResult};
use crate::repositories;
use crate::types::UserContext;
use crate::types::departments::DepartmentInput;

const RESERVED_NAME: &str = "\"Unassigned\" is reserved for users without a department";

fn require_admin(user_ctx: &UserContext) -> AdminResult<()> {
    if user_ctx.is_admin {
        Ok(())
    } else {
        Err(AdminError::Forbidden("Admin access required".to_owned()))
    }
}

/// Departments are keyed by name, so a unique violation is a caller conflict
/// rather than a server fault.
fn name_write_error(err: sqlx::Error) -> AdminError {
    match err {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            AdminError::Conflict("department name already exists".to_owned())
        },
        other => other.into(),
    }
}

fn validated_name(input: DepartmentInput) -> AdminResult<DepartmentInput> {
    let trimmed = input.name.trim();
    if trimmed.is_empty() {
        return Err(AdminError::BadRequest("name must not be empty".to_owned()));
    }
    if trimmed.eq_ignore_ascii_case("unassigned") {
        return Err(AdminError::BadRequest(RESERVED_NAME.to_owned()));
    }
    Ok(DepartmentInput {
        name: trimmed.to_owned(),
        description: input.description,
    })
}

pub(crate) async fn list_departments_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> AdminResult<Response> {
    require_admin(&user_ctx)?;
    let departments = repositories::departments::list_departments(&pool).await?;
    Ok(Json(departments).into_response())
}

pub(crate) async fn create_department_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(input): Json<DepartmentInput>,
) -> AdminResult<Response> {
    require_admin(&user_ctx)?;
    let normalized = validated_name(input)?;
    let dept = repositories::departments::create_department(&pool, &normalized)
        .await
        .map_err(name_write_error)?;
    Ok((StatusCode::CREATED, Json(dept)).into_response())
}

pub(crate) async fn update_department_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
    Json(input): Json<DepartmentInput>,
) -> AdminResult<Response> {
    require_admin(&user_ctx)?;
    let normalized = validated_name(input)?;
    let dept = repositories::departments::update_department(&pool, &id, &normalized)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdminError::NotFound("Department not found".to_owned()),
            other => name_write_error(other),
        })?;
    Ok(Json(dept).into_response())
}

pub(crate) async fn delete_department_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminResult<Response> {
    require_admin(&user_ctx)?;
    repositories::departments::delete_department(&pool, &id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdminError::NotFound("Department not found".to_owned()),
            other => other.into(),
        })?;
    Ok(StatusCode::NO_CONTENT.into_response())
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
) -> AdminResult<Response> {
    require_admin(&user_ctx)?;
    let dept_name = body.department_name.trim();
    if !dept_name.is_empty()
        && repositories::departments::find_department_by_name(&pool, dept_name)
            .await
            .inspect_err(
                |e| tracing::warn!(error = %e, dept_name, "find_department_by_name failed"),
            )
            .ok()
            .flatten()
            .is_none()
    {
        return Err(AdminError::BadRequest("unknown department".to_owned()));
    }
    repositories::departments::assign_user_to_department(&pool, &UserId::new(user_id), dept_name)
        .await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
