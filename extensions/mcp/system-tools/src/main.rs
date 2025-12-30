use anyhow::{Context, Result};
use std::{env, sync::Arc};
use system_tools::constants::{DEFAULT_MCP_PORT, DEFAULT_SERVICE_ID};
use system_tools::SystemToolsServer;
use systemprompt::identifiers::McpServerId;
use systemprompt::models::Config;
use systemprompt::profile::ProfileBootstrap;
use systemprompt::system::AppContext;
use tokio::net::TcpListener;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    ProfileBootstrap::init().context("Failed to initialize profile")?;
    Config::init().context("Failed to initialize configuration")?;

    let application_context = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );

    let service_id = if let Ok(id) = McpServerId::from_env() {
        id
    } else {
        warn!(
            default = DEFAULT_SERVICE_ID,
            "MCP_SERVICE_ID not set, using default"
        );
        McpServerId::new(DEFAULT_SERVICE_ID)
    };

    let port = if let Some(port_value) = env::var("MCP_PORT")
        .ok()
        .and_then(|port_string| port_string.parse::<u16>().ok())
    {
        port_value
    } else {
        warn!(
            default = DEFAULT_MCP_PORT,
            "MCP_PORT not set, using default"
        );
        DEFAULT_MCP_PORT
    };

    let server = SystemToolsServer::new(
        application_context.db_pool().clone(),
        service_id.clone(),
        application_context.clone(),
    )
    .context("Failed to initialize SystemToolsServer")?;

    let router = systemprompt::mcp::create_router(server, &application_context);
    let address = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&address).await?;

    info!(
        service_id = %service_id,
        address = %address,
        "System Tools MCP server listening"
    );

    axum::serve(listener, router).await?;

    Ok(())
}
