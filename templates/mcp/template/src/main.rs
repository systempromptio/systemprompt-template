use anyhow::{Context, Result};
use std::{env, sync::Arc};
use mcp_server_template::TemplateServer;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let ctx = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );
    let logger = LogService::system(ctx.db_pool().clone());

    let port = env::var("MCP_PORT")
        .unwrap_or_else(|_| "5002".to_string())
        .parse::<u16>()
        .expect("Invalid port number");

    let name = env::var("MCP_NAME").unwrap_or_else(|_| "mcp-server-template".to_string());

    let server = TemplateServer::new(ctx.db_pool().clone(), name.clone());
    let router = systemprompt_core_mcp::create_router(server, ctx.clone()).await?;
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    logger
        .info(
            "mcp_template",
            &format!("Template MCP server listening on {}", addr),
        )
        .await?;

    axum::serve(listener, router).await?;

    Ok(())
}
