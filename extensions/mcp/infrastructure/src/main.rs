use anyhow::{Context, Result};
use std::sync::Arc;
use systemprompt::credentials::CredentialsBootstrap;
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpServerId;
use systemprompt::logging;
use systemprompt::mcp;
use systemprompt::models::Config;
use systemprompt::profile::ProfileBootstrap;
use systemprompt::system::AppContext;
use systemprompt_mcp_infrastructure::InfrastructureServer;
use tokio::net::TcpListener;

const SERVICE_NAME: &str = "systemprompt-infrastructure";

#[tokio::main]
async fn main() -> Result<()> {
    ProfileBootstrap::init().context("Profile initialization failed")?;
    CredentialsBootstrap::init().context("Cloud credentials initialization failed")?;
    Config::init().context("Failed to initialize configuration")?;

    let ctx = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );

    logging::init_logging(DbPool::clone(ctx.db_pool()));

    let config = Config::get()?;
    let port = config.port;
    let service_id = McpServerId::new(SERVICE_NAME);

    let server = InfrastructureServer::new(
        DbPool::clone(ctx.db_pool()),
        service_id.clone(),
        Arc::clone(&ctx),
    );
    let router = mcp::create_router(server, &ctx);
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!(service_id = %service_id, addr = %addr, "Infrastructure MCP server listening");

    axum::serve(listener, router).await?;

    Ok(())
}
