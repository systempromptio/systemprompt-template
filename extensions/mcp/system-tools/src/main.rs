use anyhow::{Context, Result};
use std::{env, sync::Arc};
use system_tools::constants::{DEFAULT_MCP_PORT, DEFAULT_SERVICE_ID};
use system_tools::SystemToolsServer;
use systemprompt::identifiers::McpServerId;
use systemprompt::logging::LogService;
use systemprompt::models::Config;
use systemprompt::profile::ProfileBootstrap;
use systemprompt::system::AppContext;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    ProfileBootstrap::init(None).context("Failed to initialize profile")?;
    Config::init().context("Failed to initialize configuration")?;

    let application_context = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );
    let logger = LogService::system(application_context.db_pool().clone());

    let service_id = if let Ok(id) = McpServerId::from_env() {
        id
    } else {
        logger
            .warn(
                "system_tools",
                &format!("MCP_SERVICE_ID not set, using default: {DEFAULT_SERVICE_ID}"),
            )
            .await
            .ok();
        McpServerId::new(DEFAULT_SERVICE_ID)
    };

    let port = if let Some(port_value) = env::var("MCP_PORT")
        .ok()
        .and_then(|port_string| port_string.parse::<u16>().ok())
    {
        port_value
    } else {
        logger
            .warn(
                "system_tools",
                &format!("MCP_PORT not set, using default: {DEFAULT_MCP_PORT}"),
            )
            .await
            .ok();
        DEFAULT_MCP_PORT
    };

    let server = SystemToolsServer::new(
        application_context.db_pool().clone(),
        service_id.clone(),
        application_context.clone(),
    )
    .await
    .context("Failed to initialize SystemToolsServer")?;
    let router = systemprompt::mcp::create_router(server, application_context.clone()).await?;
    let address = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&address).await?;

    logger
        .info(
            service_id.as_str(),
            &format!("System Tools MCP server '{service_id}' listening on {address}"),
        )
        .await?;

    axum::serve(listener, router).await?;

    Ok(())
}
