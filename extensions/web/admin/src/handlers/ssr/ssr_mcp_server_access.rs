//! `/admin/mcp-servers/{server_id}/access` — single-MCP-server inline access panel.

use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use crate::handlers::shared;
use crate::repositories::mcp_servers;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

pub async fn mcp_server_access_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(_pool): State<Arc<PgPool>>,
    Path(server_id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    let servers = mcp_servers::list_mcp_servers(&services_path).unwrap_or_else(|e| {
        tracing::error!(error = %e, "Failed to list MCP servers");
        Vec::new()
    });
    let Some(server) = servers.into_iter().find(|s| s.id.as_str() == server_id) else {
        return (
            StatusCode::NOT_FOUND,
            Html(format!("<h1>MCP server not found: {server_id}</h1>")),
        )
            .into_response();
    };

    let data = json!({
        "page": "mcp-server-access",
        "title": format!("MCP Access · {server_id}"),
        "server_id": server_id,
        "server_type": server.server_type,
        "endpoint": server.endpoint,
    });
    super::render_page(&engine, "mcp-server-access", &data, &user_ctx, &mkt_ctx)
}
