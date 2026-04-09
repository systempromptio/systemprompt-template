use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::repositories;
use crate::types::{
    CreateMcpRequest, UpdateMcpRawYamlRequest, UpdateMcpRequest, UserContext,
};

use super::resources::get_services_path;

pub(crate) async fn list_mcp_servers_handler(
    Extension(user_ctx): Extension<UserContext>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    let servers = match repositories::list_mcp_servers(&services_path) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list MCP servers");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };
    if user_ctx.is_admin {
        return Json(servers).into_response();
    }
    let plugins =
        repositories::list_plugins_for_roles(&services_path, &user_ctx.roles).unwrap_or_else(|_| Vec::new());
    let visible_ids: std::collections::HashSet<String> = plugins
        .iter()
        .flat_map(|p| p.mcp_servers.iter().cloned())
        .collect();
    let filtered: Vec<_> = servers
        .into_iter()
        .filter(|m| visible_ids.contains(&m.id))
        .collect();
    Json(filtered).into_response()
}

pub(crate) async fn get_mcp_server_handler(Path(server_id): Path<String>) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::get_mcp_server(&services_path, &server_id) {
        Ok(Some(server)) => Json(server).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "MCP server not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get MCP server");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn create_mcp_server_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<CreateMcpRequest>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::create_mcp_server(&services_path, &body) {
        Ok(server) => {
            let name = server.id.clone();
            let sid = server.id.clone();
            let uid = user_ctx.user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_created(&uid, ActivityEntity::McpServer, &sid, &name),
                )
                .await;
            });
            (StatusCode::CREATED, Json(server)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create MCP server");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn update_mcp_server_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(server_id): Path<String>,
    Json(body): Json<UpdateMcpRequest>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::update_mcp_server(&services_path, &server_id, &body) {
        Ok(Some(server)) => {
            let name = server.id.clone();
            let sid = server_id.clone();
            let uid = user_ctx.user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_updated(&uid, ActivityEntity::McpServer, &sid, &name),
                )
                .await;
            });
            Json(server).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "MCP server not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update MCP server");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn get_mcp_server_yaml_handler(Path(server_id): Path<String>) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::get_mcp_server_raw_yaml(&services_path, &server_id) {
        Ok(Some((yaml, file_name))) => Json(serde_json::json!({
            "yaml_content": yaml,
            "file_name": file_name,
        }))
        .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "MCP server not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get MCP server YAML");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn update_mcp_server_yaml_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(server_id): Path<String>,
    Json(body): Json<UpdateMcpRawYamlRequest>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        )
            .into_response();
    }
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::update_mcp_server_raw_yaml(&services_path, &server_id, &body.yaml_content) {
        Ok(Some(server)) => {
            let name = server.id.clone();
            let sid = server_id.clone();
            let uid = user_ctx.user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_updated(&uid, ActivityEntity::McpServer, &sid, &name),
                )
                .await;
            });
            Json(server).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "MCP server not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update MCP server YAML");
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn delete_mcp_server_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(server_id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        )
            .into_response();
    }
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    if let Ok(Some(server)) = repositories::get_mcp_server(&services_path, &server_id) {
        if !server.removable {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": format!("MCP server '{}' is a system default and cannot be deleted", server_id)
                })),
            )
                .into_response();
        }
    }
    match repositories::delete_mcp_server(&services_path, &server_id) {
        Ok(true) => {
            let sid = server_id.clone();
            let uid = user_ctx.user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_deleted(&uid, ActivityEntity::McpServer, &sid, &sid),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "MCP server not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete MCP server");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
