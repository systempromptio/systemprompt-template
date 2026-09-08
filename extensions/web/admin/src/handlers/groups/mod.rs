//! JSON handlers for the groups surface: the rows themselves.
//!
//! Membership, AD mappings, marketplace entitlement and usage each have their
//! own module, because each is a different question about a group and only
//! the create/rename/delete trio is about the row.
//!
//! `unassigned` is readable everywhere and writable nowhere: it is a derived
//! membership, so renaming or deleting it would describe a set the database
//! computes rather than one an admin controls.

pub(crate) mod mappings;
pub(crate) mod marketplaces;
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
use crate::repositories::groups::{crud, marketplaces as marketplace_repo};
use crate::repositories::people_usage::DEFAULT_WINDOW_DAYS;
use crate::repositories::people_usage::breakdown::{LinkedScopeRow, list_linked_scopes};
use crate::repositories::scope::{Attribution, ScopeKind, ScopeQuery};
use crate::types::groups::{CreateGroupRequest, GroupRecord, GroupSummary, UpdateGroupRequest};

#[derive(Debug, Serialize)]
pub(crate) struct ListGroupsResponse {
    pub groups: Vec<GroupSummary>,
}

#[derive(Debug, Serialize)]
pub(crate) struct GroupDetailResponse {
    pub group: GroupRecord,
    pub member_count: i64,
    pub marketplace_ids: Vec<String>,
    pub projects: Vec<LinkedScopeRow>,
}

pub(crate) async fn list_groups_handler(State(pool): State<Arc<PgPool>>) -> AdminResult<Response> {
    let groups = crud::list_group_summaries(&pool).await?;
    Ok(Json(ListGroupsResponse { groups }).into_response())
}

pub(crate) async fn get_group_handler(
    State(pool): State<Arc<PgPool>>,
    Path(group_id): Path<String>,
) -> AdminResult<Response> {
    let group = crud::find_group(&pool, &group_id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("Group {group_id} not found")))?;
    let members = members::count_members(&pool, &group_id).await?;
    let marketplace_ids = marketplace_repo::list_group_marketplace_ids(&pool, &group_id).await?;
    let projects = list_linked_scopes(
        &pool,
        &ScopeQuery::new(
            ScopeKind::Group,
            Attribution::Member,
            &group_id,
            DEFAULT_WINDOW_DAYS,
        ),
    )
    .await?;
    Ok(Json(GroupDetailResponse {
        group,
        member_count: members,
        marketplace_ids,
        projects,
    })
    .into_response())
}

pub(crate) async fn create_group_handler(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<CreateGroupRequest>,
) -> AdminResult<Response> {
    validated_id(&body.id)?;
    if body.name.trim().is_empty() {
        return Err(AdminError::BadRequest("name must not be empty".to_owned()));
    }
    let group = crud::insert_group(&pool, &body, "dashboard").await?;
    Ok((StatusCode::CREATED, Json(group)).into_response())
}

pub(crate) async fn update_group_handler(
    State(pool): State<Arc<PgPool>>,
    Path(group_id): Path<String>,
    Json(body): Json<UpdateGroupRequest>,
) -> AdminResult<Response> {
    refuse_system_write(&pool, &group_id).await?;
    let group = crud::update_group(&pool, &group_id, &body).await?;
    Ok(Json(group).into_response())
}

pub(crate) async fn delete_group_handler(
    State(pool): State<Arc<PgPool>>,
    Path(group_id): Path<String>,
) -> AdminResult<Response> {
    crud::delete_group(&pool, &group_id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

// Why: the same shape the `groups.id` CHECK enforces, applied before the
// insert so a bad id is a 400 naming the field rather than a 500 carrying a
// constraint name.
pub(crate) fn validated_id(id: &str) -> AdminResult<()> {
    let shaped = !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
    if shaped {
        Ok(())
    } else {
        Err(AdminError::BadRequest(
            "id must be lowercase letters, digits, hyphen or underscore".to_owned(),
        ))
    }
}

// Why: entitlement is meaningful for `unassigned` — a user the directory
// placed nowhere still needs a catalogue — so this only asserts the group
// exists, where [`refuse_system_write`] also refuses the derived one.
pub(crate) async fn refuse_missing_group(pool: &PgPool, group_id: &str) -> AdminResult<()> {
    if crud::find_group(pool, group_id).await?.is_none() {
        return Err(AdminError::NotFound(format!("Group {group_id} not found")));
    }
    Ok(())
}

pub(crate) async fn refuse_system_write(pool: &PgPool, group_id: &str) -> AdminResult<GroupRecord> {
    let group = crud::find_group(pool, group_id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("Group {group_id} not found")))?;
    if group.is_system {
        return Err(AdminError::Conflict(format!(
            "Group {group_id} is a system group; its membership is derived and cannot be edited"
        )));
    }
    Ok(group)
}
