use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::types::{UpdateUserAgentRequest, UserContext};
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

pub(crate) async fn list_user_agents_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    match repositories::list_user_agents(&pool, &user_ctx.user_id).await {
        Ok(agents) => Json(json!({ "agents": agents })).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list user agents");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to list agents" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn create_user_agent_entity_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<crate::admin::types::CreateUserAgentRequest>,
) -> Response {
    match repositories::create_user_agent(&pool, &user_ctx.user_id, &req).await {
        Ok(agent) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            (StatusCode::CREATED, Json(json!(agent))).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create user agent");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create agent" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn update_user_agent_entity_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(agent_id): Path<String>,
    Json(req): Json<UpdateUserAgentRequest>,
) -> Response {
    match repositories::update_user_agent(&pool, &user_ctx.user_id, &agent_id, &req).await {
        Ok(Some(agent)) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            Json(json!(agent)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Agent not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update user agent");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to update agent" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn delete_user_agent_entity_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(agent_id): Path<String>,
) -> Response {
    match repositories::delete_user_agent(&pool, &user_ctx.user_id, &agent_id).await {
        Ok(true) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            Json(json!({ "deleted": true })).into_response()
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Agent not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete user agent");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to delete agent" })),
            )
                .into_response()
        }
    }
}
