use anyhow::{Context, Result};
use std::{env, sync::Arc};
use systemprompt::identifiers::McpServerId;
use systemprompt::models::{Config, ProfileBootstrap, SecretsBootstrap};
use systemprompt::system::AppContext;
use systemprompt_mcp_agent::SystempromptServer;
use tokio::net::TcpListener;

const DEFAULT_SERVICE_ID: &str = "systemprompt";
const DEFAULT_PORT: u16 = 5010;

#[tokio::main]
async fn main() -> Result<()> {
    systemprompt::logging::init_console_logging();

    ProfileBootstrap::init().context("Failed to initialize profile")?;
    SecretsBootstrap::init().context("Failed to initialize secrets")?;
    Config::init().context("Failed to initialize configuration")?;

    let ctx = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );

    let service_id = McpServerId::from_env().unwrap_or_else(|_| {
        tracing::warn!("MCP_SERVICE_ID not set, using default: {DEFAULT_SERVICE_ID}");
        McpServerId::new(DEFAULT_SERVICE_ID)
    });

    let port = env::var("MCP_PORT").map_or_else(
        |_| {
            tracing::warn!("MCP_PORT not set, using default: {DEFAULT_PORT}");
            DEFAULT_PORT
        },
        |p| {
            p.parse::<u16>().unwrap_or_else(|e| {
                tracing::warn!(error = %e, port = %p, "Invalid MCP_PORT, using default: {DEFAULT_PORT}");
                DEFAULT_PORT
            })
        },
    );

    let server = SystempromptServer::new(Arc::clone(ctx.db_pool()), service_id.clone())
        .context("Failed to initialize SystempromptServer")?;
    let router = systemprompt::mcp::create_router(server, ctx.db_pool());
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!(
        service_id = %service_id,
        addr = %addr,
        "SystemPrompt MCP server listening"
    );

    axum::serve(listener, router).await?;

    Ok(())
}
