//! JSON handlers for the projects surface.
//!
//! Deliberately the same shape as [`super::groups`] without sharing code with
//! it: a project has no system row, no derived membership and no marketplace
//! entitlement, so the two agree on the verbs and on nothing underneath.

pub(crate) mod mappings;
pub(crate) mod members;
pub(crate) mod usage;

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::repositories::projects::crud;
use crate::types::projects::{
    CreateProjectRequest, ProjectRow, ProjectSummary, UpdateProjectRequest,
};

#[derive(Debug, Serialize)]
pub(crate) struct ListProjectsResponse {
    pub projects: Vec<ProjectSummary>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ProjectDetailResponse {
    pub project: ProjectRow,
    pub member_count: i64,
}

pub(crate) async fn list_projects_handler(
    State(pool): State<Arc<PgPool>>,
) -> AdminResult<Response> {
    let projects = crud::list_project_summaries(&pool).await?;
    Ok(Json(ListProjectsResponse { projects }).into_response())
}

pub(crate) async fn get_project_handler(
    State(pool): State<Arc<PgPool>>,
    Path(project_id): Path<String>,
) -> AdminResult<Response> {
    let project = require_project(&pool, &project_id).await?;
    let member_count = members::count_members(&pool, &project_id).await?;
    Ok(Json(ProjectDetailResponse {
        project,
        member_count,
    })
    .into_response())
}

pub(crate) async fn create_project_handler(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<CreateProjectRequest>,
) -> AdminResult<Response> {
    super::groups::validated_id(&body.id)?;
    if body.name.trim().is_empty() {
        return Err(AdminError::BadRequest("name must not be empty".to_owned()));
    }
    let project = crud::insert_project(&pool, &body, "dashboard").await?;
    Ok((StatusCode::CREATED, Json(project)).into_response())
}

pub(crate) async fn update_project_handler(
    State(pool): State<Arc<PgPool>>,
    Path(project_id): Path<String>,
    Json(body): Json<UpdateProjectRequest>,
) -> AdminResult<Response> {
    let project = crud::update_project(&pool, &project_id, &body).await?;
    Ok(Json(project).into_response())
}

pub(crate) async fn delete_project_handler(
    State(pool): State<Arc<PgPool>>,
    Path(project_id): Path<String>,
) -> AdminResult<Response> {
    crud::delete_project(&pool, &project_id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

pub(crate) async fn require_project(pool: &PgPool, project_id: &str) -> AdminResult<ProjectRow> {
    crud::find_project(pool, project_id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("Project {project_id} not found")))
}
