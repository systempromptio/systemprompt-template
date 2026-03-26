use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::types::{
    CreateUserHookRequest, UpdateUserHookRequest, UserContext,
};
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

pub(crate) async fn list_user_hooks_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    match repositories::list_user_hooks(&pool, &user_ctx.user_id).await {
        Ok(hooks) => Json(json!({ "hooks": hooks })).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list user hooks");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to list hooks" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn create_user_hook_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<CreateUserHookRequest>,
) -> Response {
    match repositories::create_user_hook(&pool, &user_ctx.user_id, &req).await {
        Ok(hook) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            (StatusCode::CREATED, Json(json!(hook))).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create user hook");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create hook" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn update_user_hook_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(hook_id): Path<String>,
    Json(req): Json<UpdateUserHookRequest>,
) -> Response {
    match repositories::update_user_hook(&pool, &user_ctx.user_id, &hook_id, &req).await {
        Ok(Some(hook)) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            Json(json!(hook)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Hook not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update user hook");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to update hook" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn delete_user_hook_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(hook_id): Path<String>,
) -> Response {
    match repositories::delete_user_hook(&pool, &user_ctx.user_id, &hook_id).await {
        Ok(true) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            Json(json!({ "deleted": true })).into_response()
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Hook not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete user hook");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to delete hook" })),
            )
                .into_response()
        }
    }
}
