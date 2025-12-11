use anyhow::{Context, Result};
use std::{env, sync::Arc};
use systemprompt_admin::AdminServer;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;
use systemprompt_identifiers::McpServerId;
use systemprompt_models::Config;
use tokio::net::TcpListener;

/// Default service ID - MUST match the key in mcp_servers config
const DEFAULT_SERVICE_ID: &str = "systemprompt-admin";
const DEFAULT_PORT: u16 = 5002;

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

    // Read service ID from env (set by spawner) or use default
    let service_id = McpServerId::from_env().unwrap_or_else(|_| {
        eprintln!(
            "[WARN] MCP_SERVICE_ID not set, using default: {}",
            DEFAULT_SERVICE_ID
        );
        McpServerId::new(DEFAULT_SERVICE_ID)
    });

    let port = env::var("MCP_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or_else(|| {
            eprintln!("[WARN] MCP_PORT not set, using default: {}", DEFAULT_PORT);
            DEFAULT_PORT
        });

    let server = AdminServer::new(ctx.db_pool().clone(), service_id.clone(), ctx.clone());
    let router = systemprompt_core_mcp::create_router(server, ctx.clone()).await?;
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    logger
        .info(
            service_id.as_str(),
            &format!("Admin MCP server '{}' listening on {}", service_id, addr),
        )
        .await?;

    axum::serve(listener, router).await?;

    Ok(())
}
