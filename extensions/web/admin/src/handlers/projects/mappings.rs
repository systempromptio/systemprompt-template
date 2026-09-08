//! `/projects/{id}/ad-mappings` — which AD groups attribute their members to
//! this project.
//!
//! On the platform tier for the same reason the group mappings are: this is
//! what the directory grants, not what one admin arranges.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::repositories::projects::mappings as repo;
use crate::types::groups::AddAdMappingRequest;
use crate::types::projects::ProjectAdMappingRow;

use super::require_project;

#[derive(Debug, Serialize)]
pub(crate) struct ListProjectAdMappingsResponse {
    pub project_id: String,
    pub mappings: Vec<ProjectAdMappingRow>,
}

pub(crate) async fn list_project_ad_mappings_handler(
    State(pool): State<Arc<PgPool>>,
    Path(project_id): Path<String>,
) -> AdminResult<Response> {
    let mappings = repo::list_project_ad_mappings(&pool, &project_id).await?;
    Ok(Json(ListProjectAdMappingsResponse {
        project_id,
        mappings,
    })
    .into_response())
}

pub(crate) async fn add_project_ad_mapping_handler(
    State(pool): State<Arc<PgPool>>,
    Path(project_id): Path<String>,
    Json(body): Json<AddAdMappingRequest>,
) -> AdminResult<Response> {
    require_project(&pool, &project_id).await?;
    let ad_group = body.ad_group.trim();
    if ad_group.is_empty() {
        return Err(AdminError::BadRequest(
            "ad_group must not be empty".to_owned(),
        ));
    }
    repo::insert_project_ad_mapping(&pool, &project_id, ad_group, "dashboard").await?;
    Ok((StatusCode::CREATED, ()).into_response())
}

pub(crate) async fn delete_project_ad_mapping_handler(
    State(pool): State<Arc<PgPool>>,
    Path((project_id, ad_group)): Path<(String, String)>,
) -> AdminResult<Response> {
    repo::delete_project_ad_mapping(&pool, &project_id, &ad_group).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}
