//! `/projects/{id}/members` — who is attributed to a project.
//!
//! The same two-writer rule groups have: the directory replaces its own rows
//! at every sign-in, so only the manual half can be removed here and a
//! directory-sourced member is refused with a 409 pointing at AD.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::AdminResult;
use crate::repositories::projects::members as repo;
use crate::repositories::scope::defaults;
use crate::types::UserContext;
use crate::types::projects::{AddProjectMemberRequest, ProjectMemberRow};

use super::require_project;

#[derive(Debug, Serialize)]
pub(crate) struct ListProjectMembersResponse {
    pub project_id: String,
    pub members: Vec<ProjectMemberRow>,
}

pub(crate) async fn list_project_members_handler(
    State(pool): State<Arc<PgPool>>,
    Path(project_id): Path<String>,
) -> AdminResult<Response> {
    let members = repo::list_project_members(&pool, &project_id).await?;
    Ok(Json(ListProjectMembersResponse {
        project_id,
        members,
    })
    .into_response())
}

pub(crate) async fn add_project_member_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(project_id): Path<String>,
    Json(body): Json<AddProjectMemberRequest>,
) -> AdminResult<Response> {
    require_project(&pool, &project_id).await?;
    repo::insert_project_member(&pool, &project_id, &body.user_id, &user_ctx.user_id).await?;
    defaults::recompute_scope_defaults(&pool).await?;
    Ok((StatusCode::CREATED, ()).into_response())
}

pub(crate) async fn remove_project_member_handler(
    State(pool): State<Arc<PgPool>>,
    Path((project_id, user_id)): Path<(String, String)>,
) -> AdminResult<Response> {
    repo::delete_project_member(&pool, &project_id, &UserId::new(user_id)).await?;
    defaults::recompute_scope_defaults(&pool).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

pub(crate) async fn count_members(pool: &PgPool, project_id: &str) -> AdminResult<i64> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(DISTINCT user_id)::BIGINT AS "count!"
           FROM project_members WHERE project_id = $1"#,
        project_id
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}
