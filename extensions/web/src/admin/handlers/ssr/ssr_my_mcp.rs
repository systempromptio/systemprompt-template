use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

pub(crate) async fn my_mcp_servers_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let servers = repositories::list_user_mcp_servers(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list user MCP servers");
            vec![]
        });

    let server_count = servers.len();
    let enabled_count = servers.iter().filter(|s| s.enabled).count();

    let servers_json: Vec<serde_json::Value> = servers
        .iter()
        .map(|s| {
            json!({
                "id": s.id,
                "mcp_server_id": s.mcp_server_id,
                "name": s.name,
                "description": s.description,
                "binary": s.binary,
                "package_name": s.package_name,
                "port": s.port,
                "endpoint": s.endpoint,
                "enabled": s.enabled,
                "oauth_required": s.oauth_required,
                "oauth_scopes": s.oauth_scopes,
                "oauth_audience": s.oauth_audience,
                "base_mcp_server_id": s.base_mcp_server_id,
                "is_forked": s.base_mcp_server_id.is_some(),
                "created_at": s.created_at,
                "updated_at": s.updated_at,
            })
        })
        .collect();

    let data = json!({
        "page": "my-mcp-servers",
        "title": "My MCP Servers",
        "mcp_servers": servers_json,
        "server_count": server_count,
        "enabled_count": enabled_count,
    });
    super::render_page(&engine, "my-mcp-servers", &data, &user_ctx, &mkt_ctx)
}

pub(crate) async fn my_mcp_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let server_id = params.get("id");
    let is_edit = server_id.is_some();

    let server = if let Some(id) = server_id {
        let servers = repositories::list_user_mcp_servers(&pool, &user_ctx.user_id)
            .await
            .unwrap_or_default();
        servers.into_iter().find(|s| s.mcp_server_id == *id)
    } else {
        None
    };

    let is_forked = server
        .as_ref()
        .is_some_and(|s| s.base_mcp_server_id.is_some());

    let scopes_csv = server
        .as_ref()
        .map_or(String::new(), |s| s.oauth_scopes.join(", "));

    let data = json!({
        "page": "my-mcp-edit",
        "title": if is_edit { "Edit My MCP Server" } else { "Create My MCP Server" },
        "is_edit": is_edit,
        "server": server,
        "is_forked": is_forked,
        "scopes_csv": scopes_csv,
    });
    super::render_page(&engine, "my-mcp-edit", &data, &user_ctx, &mkt_ctx)
}
