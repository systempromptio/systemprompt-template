use axum::{extract::State, response::IntoResponse, routing::get, Router};
use serde::{Deserialize, Serialize};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::api::ApiError;
use systemprompt_core_system::{AppContext, CollectionResponse};

use crate::services::registry::manager::RegistryService;

#[derive(Debug, Serialize, Deserialize)]
pub struct McpRegistryServer {
    pub name: String,
    pub version: String,
    pub description: String,
    pub port: u16,
    pub enabled: bool,
    pub display_in_web: bool,
    pub oauth_required: bool,
    pub oauth_scopes: Vec<String>,
    pub endpoint: String,
    pub status: String,
}

pub async fn handle_mcp_registry(State(ctx): State<AppContext>) -> impl IntoResponse {
    let logger = LogService::system(ctx.db_pool().clone());

    // Load MCP servers from services.yaml
    let server_configs = match RegistryService::get_enabled_servers_as_config().await {
        Ok(configs) => configs,
        Err(e) => {
            logger
                .error(
                    "mcp_api",
                    &format!("Failed to load MCP server configs: {e}"),
                )
                .await
                .ok();
            return ApiError::internal_error(format!("Failed to retrieve MCP registry: {e}"))
                .into_response();
        },
    };

    let api_external_url = &ctx.config().api_external_url;
    let servers: Vec<McpRegistryServer> = server_configs
        .iter()
        .map(|config| McpRegistryServer {
            name: config.name.clone(),
            version: config.version.clone(),
            description: config.description.clone(),
            port: config.port,
            enabled: config.enabled,
            display_in_web: config.display_in_web,
            oauth_required: config.oauth.required,
            oauth_scopes: config.oauth.scopes.iter().map(std::string::ToString::to_string).collect(),
            endpoint: format!("{}/api/v1/mcp/{}/mcp", api_external_url, config.name),
            status: if config.enabled {
                "enabled".to_string()
            } else {
                "disabled".to_string()
            },
        })
        .collect();

    CollectionResponse::new(servers).into_response()
}

pub fn router(ctx: &AppContext) -> Router {
    Router::new()
        .route("/", get(handle_mcp_registry))
        .with_state(ctx.clone())
}
