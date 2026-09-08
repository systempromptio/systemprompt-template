//! `/groups/{id}/members` — who is in a group, and the manual half of it.
//!
//! Reading works for every group including `unassigned`, whose membership the
//! `user_groups` view derives. Writing does not: a derived member has no row
//! to add or remove, and a directory-sourced one would reappear at the next
//! sign-in, so both are refused with a 409 that says which.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::AdminResult;
use crate::repositories::groups::members as repo;
use crate::repositories::scope::defaults;
use crate::types::UserContext;
use crate::types::groups::{AddGroupMemberRequest, GroupMemberRow};

use super::refuse_system_write;

#[derive(Debug, Serialize)]
pub(crate) struct ListGroupMembersResponse {
    pub group_id: String,
    pub members: Vec<GroupMemberRow>,
}

pub(crate) async fn list_group_members_handler(
    State(pool): State<Arc<PgPool>>,
    Path(group_id): Path<String>,
) -> AdminResult<Response> {
    let members = repo::list_group_members(&pool, &group_id).await?;
    Ok(Json(ListGroupMembersResponse { group_id, members }).into_response())
}

pub(crate) async fn add_group_member_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(group_id): Path<String>,
    Json(body): Json<AddGroupMemberRequest>,
) -> AdminResult<Response> {
    refuse_system_write(&pool, &group_id).await?;
    repo::insert_group_member(&pool, &group_id, &body.user_id, &user_ctx.user_id).await?;
    defaults::recompute_scope_defaults(&pool).await?;
    Ok((StatusCode::CREATED, ()).into_response())
}

pub(crate) async fn remove_group_member_handler(
    State(pool): State<Arc<PgPool>>,
    Path((group_id, user_id)): Path<(String, String)>,
) -> AdminResult<Response> {
    refuse_system_write(&pool, &group_id).await?;
    repo::delete_group_member(&pool, &group_id, &UserId::new(user_id)).await?;
    defaults::recompute_scope_defaults(&pool).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

pub(crate) async fn count_members(pool: &PgPool, group_id: &str) -> AdminResult<i64> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(DISTINCT user_id)::BIGINT AS "count!"
           FROM user_groups WHERE group_id = $1"#,
        group_id
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}
