use std::sync::Arc;

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::repositories;
use crate::types::{UpdateUserSkillRequest, UserContext};
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use systemprompt::identifiers::SkillId;

use super::plugins::is_entity_in_platform_plugin;
use crate::handlers::{responses::SkillsListResponse, shared};

pub async fn list_user_skills_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    match repositories::list_user_skills(&pool, &user_ctx.user_id).await {
        Ok(skills) => Json(SkillsListResponse { skills }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list user skills");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to list skills")
        }
    }
}

pub async fn create_user_skill_handler(
    Extension(user_ctx): Extension<UserContext>,
    Extension(tier_cache): Extension<crate::tier_enforcement::TierEnforcementCache>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<crate::types::CreateSkillRequest>,
) -> Response {
    let limit_check = crate::tier_enforcement::check_limit(
        &tier_cache,
        pool.as_ref(),
        &user_ctx.user_id,
        crate::tier_limits::LimitCheck::CreateSkill,
    )
    .await;
    if !limit_check.allowed {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "entity_limit_reached",
                "entity_type": "skill",
                "message": limit_check.reason,
                "limit": limit_check.limit_value,
                "current": limit_check.current_value,
            })),
        )
            .into_response();
    }

    match repositories::create_user_skill(&pool, &user_ctx.user_id, &req).await {
        Ok(skill) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let id = skill.id.clone();
            let name = skill.name.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_created(&uid, ActivityEntity::UserSkill, &id, &name),
                )
                .await;
            });
            (StatusCode::CREATED, Json(skill)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create user skill");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create skill")
        }
    }
}

pub async fn update_user_skill_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(skill_id): Path<String>,
    Json(req): Json<UpdateUserSkillRequest>,
) -> Response {
    let skill_id_typed = SkillId::from(skill_id.clone());
    if is_entity_in_platform_plugin(&pool, &user_ctx.user_id, &skill_id, "skill").await {
        return shared::error_response(StatusCode::FORBIDDEN, "Platform skill cannot be modified");
    }
    match repositories::update_user_skill(&pool, &user_ctx.user_id, &skill_id_typed, &req).await {
        Ok(Some(skill)) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let id = skill.id.clone();
            let name = skill.name.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_updated(&uid, ActivityEntity::UserSkill, &id, &name),
                )
                .await;
            });
            Json(skill).into_response()
        }
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "Skill not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update user skill");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to update skill")
        }
    }
}

pub async fn delete_user_skill_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(skill_id): Path<String>,
) -> Response {
    let skill_id_typed = SkillId::from(skill_id.clone());
    if is_entity_in_platform_plugin(&pool, &user_ctx.user_id, &skill_id, "skill").await {
        return shared::error_response(StatusCode::FORBIDDEN, "Platform skill cannot be modified");
    }
    match repositories::delete_user_skill(&pool, &user_ctx.user_id, &skill_id_typed).await {
        Ok(true) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let id = skill_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_deleted(&uid, ActivityEntity::UserSkill, &id, &id),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "Skill not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete user skill");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete skill")
        }
    }
}
