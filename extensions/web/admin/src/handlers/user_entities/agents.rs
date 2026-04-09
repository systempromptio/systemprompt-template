use std::sync::Arc;

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::repositories;
use crate::types::{UpdateUserAgentRequest, UserContext};
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use systemprompt::identifiers::AgentId;

use super::plugins::is_entity_in_platform_plugin;
use crate::handlers::{responses::AgentsListResponse, shared};

pub async fn list_user_agents_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    match repositories::list_user_agents(&pool, &user_ctx.user_id).await {
        Ok(agents) => Json(AgentsListResponse { agents }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list user agents");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to list agents")
        }
    }
}

pub async fn create_user_agent_entity_handler(
    Extension(user_ctx): Extension<UserContext>,
    Extension(tier_cache): Extension<crate::tier_enforcement::TierEnforcementCache>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<crate::types::CreateUserAgentRequest>,
) -> Response {
    let limit_check = crate::tier_enforcement::check_limit(
        &tier_cache,
        pool.as_ref(),
        &user_ctx.user_id,
        crate::tier_limits::LimitCheck::CreateAgent,
    )
    .await;
    if !limit_check.allowed {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "entity_limit_reached",
                "entity_type": "agent",
                "message": limit_check.reason,
                "limit": limit_check.limit_value,
                "current": limit_check.current_value,
            })),
        )
            .into_response();
    }

    match repositories::create_user_agent(&pool, &user_ctx.user_id, &req).await {
        Ok(agent) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let id = agent.id.clone();
            let name = agent.name.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_created(&uid, ActivityEntity::UserAgent, &id, &name),
                )
                .await;
            });
            (StatusCode::CREATED, Json(agent)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create user agent");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create agent")
        }
    }
}

pub async fn update_user_agent_entity_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(agent_id): Path<String>,
    Json(req): Json<UpdateUserAgentRequest>,
) -> Response {
    let agent_id_typed = AgentId::from(agent_id.clone());
    if is_entity_in_platform_plugin(&pool, &user_ctx.user_id, &agent_id, "agent").await {
        return shared::error_response(StatusCode::FORBIDDEN, "Platform agent cannot be modified");
    }
    match repositories::update_user_agent(&pool, &user_ctx.user_id, &agent_id_typed, &req).await {
        Ok(Some(agent)) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let id = agent.id.clone();
            let name = agent.name.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_updated(&uid, ActivityEntity::UserAgent, &id, &name),
                )
                .await;
            });
            Json(agent).into_response()
        }
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "Agent not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update user agent");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to update agent")
        }
    }
}

pub async fn delete_user_agent_entity_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(agent_id): Path<String>,
) -> Response {
    let agent_id_typed = AgentId::from(agent_id.clone());
    if is_entity_in_platform_plugin(&pool, &user_ctx.user_id, &agent_id, "agent").await {
        return shared::error_response(StatusCode::FORBIDDEN, "Platform agent cannot be modified");
    }
    match repositories::delete_user_agent(&pool, &user_ctx.user_id, &agent_id_typed).await {
        Ok(true) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let id = agent_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_deleted(&uid, ActivityEntity::UserAgent, &id, &id),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "Agent not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete user agent");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete agent")
        }
    }
}
