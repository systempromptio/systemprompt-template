use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::types::{
    CreateUserMcpServerRequest, UpdateUserMcpServerRequest, UserContext,
};
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

pub(crate) async fn list_user_mcp_servers_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    match repositories::list_user_mcp_servers(&pool, &user_ctx.user_id).await {
        Ok(servers) => Json(json!({ "mcp_servers": servers })).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list user MCP servers");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to list MCP servers" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn create_user_mcp_server_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<CreateUserMcpServerRequest>,
) -> Response {
    match repositories::create_user_mcp_server(&pool, &user_ctx.user_id, &req).await {
        Ok(server) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            (StatusCode::CREATED, Json(json!(server))).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create user MCP server");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create MCP server" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn update_user_mcp_server_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(server_id): Path<String>,
    Json(req): Json<UpdateUserMcpServerRequest>,
) -> Response {
    match repositories::update_user_mcp_server(&pool, &user_ctx.user_id, &server_id, &req).await {
        Ok(Some(server)) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            Json(json!(server)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "MCP server not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update user MCP server");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to update MCP server" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn delete_user_mcp_server_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(server_id): Path<String>,
) -> Response {
    match repositories::delete_user_mcp_server(&pool, &user_ctx.user_id, &server_id).await {
        Ok(true) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            Json(json!({ "deleted": true })).into_response()
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "MCP server not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete user MCP server");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to delete MCP server" })),
            )
                .into_response()
        }
    }
}
