//! `/groups/{id}/marketplaces` — the plugin catalogue this group may see.
//!
//! The stored form is an access-control rule per marketplace, but the screen
//! asks a set question, so the PUT takes the whole set and the repository
//! reconciles: rules for listed marketplaces are upserted to `allow`, and
//! this group's rules on unlisted ones are deleted. A marketplace's own
//! `default_included` is never touched here.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;

use crate::error::AdminResult;
use crate::repositories::groups::marketplaces as repo;
use crate::types::groups::SetGroupMarketplacesRequest;

#[derive(Debug, Serialize)]
pub(crate) struct GroupMarketplacesResponse {
    pub group_id: String,
    pub marketplace_ids: Vec<String>,
}

pub(crate) async fn list_group_marketplaces_handler(
    State(pool): State<Arc<PgPool>>,
    Path(group_id): Path<String>,
) -> AdminResult<Response> {
    let marketplace_ids = repo::list_group_marketplace_ids(&pool, &group_id).await?;
    Ok(Json(GroupMarketplacesResponse {
        group_id,
        marketplace_ids,
    })
    .into_response())
}

pub(crate) async fn set_group_marketplaces_handler(
    State(pool): State<Arc<PgPool>>,
    Path(group_id): Path<String>,
    Json(body): Json<SetGroupMarketplacesRequest>,
) -> AdminResult<Response> {
    super::refuse_missing_group(&pool, &group_id).await?;
    let marketplace_ids =
        repo::set_group_marketplaces(&pool, &group_id, &body.marketplace_ids).await?;
    Ok(Json(GroupMarketplacesResponse {
        group_id,
        marketplace_ids,
    })
    .into_response())
}
