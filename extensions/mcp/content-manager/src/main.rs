use anyhow::{Context, Result};
use std::{env, sync::Arc};
use systemprompt::identifiers::McpServerId;
use systemprompt::models::{Config, ProfileBootstrap, SecretsBootstrap};
use systemprompt::system::AppContext;
use systemprompt_mcp_content_manager::ContentManagerServer;
use tokio::net::TcpListener;

const DEFAULT_SERVICE_ID: &str = "content-manager";
const DEFAULT_PORT: u16 = 5040;

#[tokio::main]
async fn main() -> Result<()> {
    ProfileBootstrap::init().context("Failed to initialize profile")?;
    SecretsBootstrap::init().context("Failed to initialize secrets")?;
    Config::init().context("Failed to initialize configuration")?;

    let ctx = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );

    systemprompt::logging::init_logging(ctx.db_pool().clone());

    let service_id = McpServerId::from_env().unwrap_or_else(|_| {
        tracing::warn!("MCP_SERVICE_ID not set, using default: {DEFAULT_SERVICE_ID}");
        McpServerId::new(DEFAULT_SERVICE_ID)
    });

    let port = if let Ok(p) = env::var("MCP_PORT") {
        p.parse::<u16>().unwrap_or_else(|e| {
            tracing::warn!(error = %e, port = %p, "Invalid MCP_PORT, using default: {DEFAULT_PORT}");
            DEFAULT_PORT
        })
    } else {
        tracing::warn!("MCP_PORT not set, using default: {DEFAULT_PORT}");
        DEFAULT_PORT
    };

    let server = ContentManagerServer::new(ctx.db_pool().clone(), service_id.clone(), ctx.clone())
        .await
        .context("Failed to initialize ContentManagerServer")?;

    let router = systemprompt::mcp::create_router(server, ctx.db_pool());
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!(
        service_id = %service_id,
        addr = %addr,
        "Content Manager MCP server listening"
    );

    axum::serve(listener, router).await?;

    Ok(())
}
