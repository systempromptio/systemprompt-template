use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::types::{UpdateUserSkillRequest, UserContext};
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

pub(crate) async fn list_user_skills_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    match repositories::list_user_skills(&pool, &user_ctx.user_id).await {
        Ok(skills) => Json(json!({ "skills": skills })).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list user skills");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to list skills" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn create_user_skill_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<crate::admin::types::CreateSkillRequest>,
) -> Response {
    match repositories::create_user_skill(&pool, &user_ctx.user_id, &req).await {
        Ok(skill) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            (StatusCode::CREATED, Json(json!(skill))).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create user skill");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create skill" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn update_user_skill_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(skill_id): Path<String>,
    Json(req): Json<UpdateUserSkillRequest>,
) -> Response {
    match repositories::update_user_skill(&pool, &user_ctx.user_id, &skill_id, &req).await {
        Ok(Some(skill)) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            Json(json!(skill)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Skill not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update user skill");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to update skill" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn delete_user_skill_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(skill_id): Path<String>,
) -> Response {
    match repositories::delete_user_skill(&pool, &user_ctx.user_id, &skill_id).await {
        Ok(true) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            Json(json!({ "deleted": true })).into_response()
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Skill not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete user skill");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to delete skill" })),
            )
                .into_response()
        }
    }
}
