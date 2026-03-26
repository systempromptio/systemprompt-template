use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::types::{CreateUserPluginRequest, UpdateUserPluginRequest, UserContext};
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

pub(crate) async fn list_user_plugins_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    match repositories::list_user_plugins(&pool, &user_ctx.user_id).await {
        Ok(plugins) => Json(json!({ "plugins": plugins })).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list user plugins");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to list plugins" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn create_user_plugin_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<CreateUserPluginRequest>,
) -> Response {
    match repositories::create_user_plugin(&pool, &user_ctx.user_id, &req).await {
        Ok(plugin) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            (StatusCode::CREATED, Json(json!(plugin))).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create user plugin");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create plugin" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn update_user_plugin_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    Json(req): Json<UpdateUserPluginRequest>,
) -> Response {
    match repositories::update_user_plugin(&pool, &user_ctx.user_id, &plugin_id, &req).await {
        Ok(Some(plugin)) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            Json(json!(plugin)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Plugin not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update user plugin");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to update plugin" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn delete_user_plugin_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
) -> Response {
    match repositories::delete_user_plugin(&pool, &user_ctx.user_id, &plugin_id).await {
        Ok(true) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            Json(json!({ "deleted": true })).into_response()
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Plugin not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete user plugin");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to delete plugin" })),
            )
                .into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SetAssociationsRequest {
    pub ids: Vec<String>,
}

pub(crate) async fn set_plugin_skills_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    Json(req): Json<SetAssociationsRequest>,
) -> Response {
    let plugin = repositories::get_user_plugin(&pool, &user_ctx.user_id, &plugin_id).await;
    match plugin {
        Ok(Some(p)) => {
            if let Err(e) = repositories::set_plugin_skills(&pool, &p.id, &req.ids).await {
                tracing::error!(error = %e, "Failed to set plugin skills");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to set skills" })),
                )
                    .into_response();
            }
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            Json(json!({ "ok": true })).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Plugin not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get user plugin");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal error" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn set_plugin_agents_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    Json(req): Json<SetAssociationsRequest>,
) -> Response {
    let plugin = repositories::get_user_plugin(&pool, &user_ctx.user_id, &plugin_id).await;
    match plugin {
        Ok(Some(p)) => {
            if let Err(e) = repositories::set_plugin_agents(&pool, &p.id, &req.ids).await {
                tracing::error!(error = %e, "Failed to set plugin agents");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to set agents" })),
                )
                    .into_response();
            }
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            Json(json!({ "ok": true })).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Plugin not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get user plugin");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal error" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn set_plugin_mcp_servers_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    Json(req): Json<SetAssociationsRequest>,
) -> Response {
    let plugin = repositories::get_user_plugin(&pool, &user_ctx.user_id, &plugin_id).await;
    match plugin {
        Ok(Some(p)) => {
            if let Err(e) = repositories::set_plugin_mcp_servers(&pool, &p.id, &req.ids).await {
                tracing::error!(error = %e, "Failed to set plugin MCP servers");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to set MCP servers" })),
                )
                    .into_response();
            }
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            Json(json!({ "ok": true })).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Plugin not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get user plugin");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal error" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn set_plugin_hooks_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    Json(req): Json<SetAssociationsRequest>,
) -> Response {
    let plugin = repositories::get_user_plugin(&pool, &user_ctx.user_id, &plugin_id).await;
    match plugin {
        Ok(Some(p)) => {
            if let Err(e) = repositories::set_plugin_hooks(&pool, &p.id, &req.ids).await {
                tracing::error!(error = %e, "Failed to set plugin hooks");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to set hooks" })),
                )
                    .into_response();
            }
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            Json(json!({ "ok": true })).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Plugin not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get user plugin");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal error" })),
            )
                .into_response()
        }
    }
}
