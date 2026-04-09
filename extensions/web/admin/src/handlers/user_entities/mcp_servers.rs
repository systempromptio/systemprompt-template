use std::sync::Arc;

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::repositories;
use crate::types::{CreateUserMcpServerRequest, UpdateUserMcpServerRequest, UserContext};
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use sqlx::PgPool;

use systemprompt::identifiers::McpServerId;

use crate::handlers::{responses::McpServersListResponse, shared};

#[derive(Debug, Deserialize)]
pub struct CreateMcpServerApiRequest {
    pub mcp_server_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub endpoint: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMcpServerApiRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub endpoint: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SetPluginMcpServersRequest {
    pub ids: Vec<String>,
}

pub async fn list_user_mcp_servers_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    match repositories::user_mcp_servers::list_user_mcp_servers(&pool, &user_ctx.user_id).await {
        Ok(servers) => Json(McpServersListResponse {
            mcp_servers: servers,
        })
        .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list user MCP servers");
            shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to list MCP servers",
            )
        }
    }
}

pub async fn create_user_mcp_server_handler(
    Extension(user_ctx): Extension<UserContext>,
    Extension(tier_cache): Extension<crate::tier_enforcement::TierEnforcementCache>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<CreateMcpServerApiRequest>,
) -> Response {
    let limit_check = crate::tier_enforcement::check_limit(
        &tier_cache,
        pool.as_ref(),
        &user_ctx.user_id,
        crate::tier_limits::LimitCheck::CreateMcpServer,
    )
    .await;
    if !limit_check.allowed {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "entity_limit_reached",
                "entity_type": "mcp_server",
                "message": limit_check.reason,
                "limit": limit_check.limit_value,
                "current": limit_check.current_value,
            })),
        )
            .into_response();
    }

    let create_req = CreateUserMcpServerRequest {
        mcp_server_id: McpServerId::new(req.mcp_server_id),
        name: req.name,
        description: req.description,
        binary: String::new(),
        package_name: String::new(),
        port: 0,
        endpoint: req.endpoint,
        oauth_required: false,
        oauth_scopes: vec![],
        oauth_audience: String::new(),
        base_mcp_server_id: None,
    };
    match repositories::user_mcp_servers::create_user_mcp_server(
        &pool,
        &user_ctx.user_id,
        &create_req,
    )
    .await
    {
        Ok(_server) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let id = create_req.mcp_server_id.to_string();
            let name = create_req.name.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_created(&uid, ActivityEntity::McpServer, &id, &name),
                )
                .await;
            });
            StatusCode::CREATED.into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create MCP server");
            shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create MCP server",
            )
        }
    }
}

pub async fn update_user_mcp_server_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(mcp_server_id): Path<String>,
    Json(req): Json<UpdateMcpServerApiRequest>,
) -> Response {
    let update_req = UpdateUserMcpServerRequest {
        name: req.name,
        description: req.description,
        binary: None,
        package_name: None,
        port: None,
        endpoint: req.endpoint,
        enabled: req.enabled,
        oauth_required: None,
        oauth_scopes: None,
        oauth_audience: None,
    };
    let mcp_server_id_typed = McpServerId::new(mcp_server_id.clone());
    match repositories::user_mcp_servers::update_user_mcp_server(
        &pool,
        &user_ctx.user_id,
        &mcp_server_id_typed,
        &update_req,
    )
    .await
    {
        Ok(Some(_)) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let id = mcp_server_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_updated(&uid, ActivityEntity::McpServer, &id, &id),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "MCP server not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update MCP server");
            shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update MCP server",
            )
        }
    }
}

pub async fn delete_user_mcp_server_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(mcp_server_id): Path<String>,
) -> Response {
    let mcp_server_id_typed = McpServerId::new(mcp_server_id.clone());
    match repositories::user_mcp_servers::delete_user_mcp_server(
        &pool,
        &user_ctx.user_id,
        &mcp_server_id_typed,
    )
    .await
    {
        Ok(true) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let id = mcp_server_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_deleted(&uid, ActivityEntity::McpServer, &id, &id),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "MCP server not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete MCP server");
            shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete MCP server",
            )
        }
    }
}

pub async fn set_plugin_mcp_servers_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    Json(req): Json<SetPluginMcpServersRequest>,
) -> Response {
    if let Err(resp) =
        update_plugin_mcp_servers(&pool, &user_ctx.user_id, &plugin_id, &req.ids).await
    {
        return resp;
    }
    mark_dirty_and_record_mcp_activity(&pool, &user_ctx.user_id, &plugin_id).await;
    StatusCode::NO_CONTENT.into_response()
}

async fn update_plugin_mcp_servers(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
    plugin_id: &str,
    ids: &[String],
) -> Result<(), Response> {
    let plugin = match repositories::find_user_plugin(pool, user_id, plugin_id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Err(shared::error_response(StatusCode::NOT_FOUND, "Plugin not found"));
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to get user plugin");
            return Err(shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal error"));
        }
    };

    let mcp_ids: Vec<McpServerId> = ids
        .iter()
        .map(|s| McpServerId::new(s.clone()))
        .collect();
    if let Err(e) =
        repositories::user_plugins::set_plugin_mcp_servers(pool, &plugin.id, &mcp_ids).await
    {
        tracing::error!(error = %e, "Failed to set plugin MCP servers");
        return Err(shared::error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to set MCP servers",
        ));
    }
    Ok(())
}

async fn mark_dirty_and_record_mcp_activity(pool: &Arc<PgPool>, user_id: &systemprompt::identifiers::UserId, plugin_id: &str) {
    if let Err(e) = repositories::mark_user_dirty(pool, user_id).await {
        tracing::warn!(error = %e, "Failed to mark user dirty");
    }
    let activity_pool = Arc::clone(pool);
    let uid = user_id.clone();
    let pid = plugin_id.to_string();
    tokio::spawn(async move {
        activity::record(
            &activity_pool,
            NewActivity::entity_updated(&uid, ActivityEntity::Plugin, &pid, &pid),
        )
        .await;
    });
}
