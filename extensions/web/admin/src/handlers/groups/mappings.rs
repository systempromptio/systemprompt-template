//! `/groups/{id}/ad-mappings` — which AD groups project onto this group.
//!
//! Reads sit with the other group reads; the writes sit on the platform tier
//! alone. A mapping decides what the directory grants everyone who holds that
//! AD group, so moving one is the same class of act as granting
//! `platform_admin`, not the same class as adding a member by hand.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::repositories::groups::{crud, mappings as repo};
use crate::types::groups::{AddAdMappingRequest, GroupAdMappingRow};

#[derive(Debug, Serialize)]
pub(crate) struct ListAdMappingsResponse {
    pub group_id: String,
    pub mappings: Vec<GroupAdMappingRow>,
}

pub(crate) async fn list_group_ad_mappings_handler(
    State(pool): State<Arc<PgPool>>,
    Path(group_id): Path<String>,
) -> AdminResult<Response> {
    let mappings = repo::list_group_ad_mappings(&pool, &group_id).await?;
    Ok(Json(ListAdMappingsResponse { group_id, mappings }).into_response())
}

pub(crate) async fn add_group_ad_mapping_handler(
    State(pool): State<Arc<PgPool>>,
    Path(group_id): Path<String>,
    Json(body): Json<AddAdMappingRequest>,
) -> AdminResult<Response> {
    require_group(&pool, &group_id).await?;
    let ad_group = body.ad_group.trim();
    if ad_group.is_empty() {
        return Err(AdminError::BadRequest(
            "ad_group must not be empty".to_owned(),
        ));
    }
    repo::insert_group_ad_mapping(&pool, &group_id, ad_group, "dashboard").await?;
    Ok((StatusCode::CREATED, ()).into_response())
}

pub(crate) async fn delete_group_ad_mapping_handler(
    State(pool): State<Arc<PgPool>>,
    Path((group_id, ad_group)): Path<(String, String)>,
) -> AdminResult<Response> {
    repo::delete_group_ad_mapping(&pool, &group_id, &ad_group).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

async fn require_group(pool: &PgPool, group_id: &str) -> AdminResult<()> {
    if crud::find_group(pool, group_id).await?.is_none() {
        return Err(AdminError::NotFound(format!("Group {group_id} not found")));
    }
    Ok(())
}
