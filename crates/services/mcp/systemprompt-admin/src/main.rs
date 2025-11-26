use anyhow::{Context, Result};
use std::{env, sync::Arc};
use systemprompt_admin::AdminServer;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;
use systemprompt_models::Config;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    Config::init().context("Failed to initialize configuration")?;

    let ctx = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );
    let logger = LogService::system(ctx.db_pool().clone());

    let port = env::var("MCP_PORT")
        .unwrap_or_else(|_| "5002".to_string())
        .parse::<u16>()
        .context("Invalid MCP_PORT: must be a valid port number")?;

    let name = env::var("MCP_NAME").unwrap_or_else(|_| "systemprompt-admin".to_string());

    let server = AdminServer::new(ctx.db_pool().clone(), name.clone());
    let router = systemprompt_core_mcp::create_router(server, ctx.clone()).await?;
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    logger
        .info(
            "mcp_admin",
            &format!("Admin MCP server listening on {}", addr),
        )
        .await?;

    axum::serve(listener, router).await?;

    Ok(())
}
