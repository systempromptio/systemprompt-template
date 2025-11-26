pub mod api;
pub mod cli;
pub mod middleware;
pub mod models;
pub mod repository;
pub mod services;

pub use systemprompt_models::mcp::{
    Deployment, DeploymentConfig, McpServerConfig, McpServerInfo, OAuthRequirement, Package,
    Remote, Repository, ServerManifest, Settings, UserContext, ERROR, RUNNING, STARTING, STOPPED,
};

pub use services::registry::McpServerRegistry;
pub use services::{McpManager, ServiceManager};

/// MCP Protocol Version from rmcp specification
pub fn mcp_protocol_version() -> String {
    ProtocolVersion::LATEST.to_string()
}

pub mod registry {
    pub use crate::services::registry::export::export_registry_servers;
}

pub use cli::{list_services, show_status, start_services, stop_services, McpServiceDisplay};

pub use rmcp::model::ProtocolVersion;
use rmcp::transport::{streamable_http_server::StreamableHttpServerConfig, StreamableHttpService};
use rmcp::ServerHandler;
use std::sync::Arc;
use std::time::Duration;
use systemprompt_core_system::AppContext;

use crate::middleware::DatabaseSessionManager;

pub async fn create_router<S>(
    server: S,
    app_context: Arc<AppContext>,
) -> anyhow::Result<axum::Router>
where
    S: ServerHandler + Clone + Send + Sync + 'static,
{
    let config = StreamableHttpServerConfig {
        stateful_mode: true,
        sse_keep_alive: Some(Duration::from_secs(30)),
    };

    let session_manager = DatabaseSessionManager::new(app_context.db_pool().clone());

    let service =
        StreamableHttpService::new(move || Ok(server.clone()), session_manager.into(), config);

    Ok(axum::Router::new().nest_service("/mcp", service))
}
