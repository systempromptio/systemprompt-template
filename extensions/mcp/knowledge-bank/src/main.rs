//! Entry point for the knowledge-bank MCP server binary.

use anyhow::{Context, Result};
use std::env;
use std::sync::Arc;
use systemprompt::config::{ProfileBootstrap, SecretsBootstrap, init_config};
use systemprompt::identifiers::McpServerId;
use systemprompt::system::AppContext;
use systemprompt_mcp_knowledge_bank::{KnowledgeBankServer, StubRetriever};
use tokio::net::TcpListener;

const DEFAULT_SERVICE_ID: &str = "knowledge-bank";
const DEFAULT_PORT: u16 = 5030;

#[tokio::main]
async fn main() -> Result<()> {
    systemprompt::logging::init_console_logging();

    ProfileBootstrap::init().context("Failed to initialize profile")?;
    SecretsBootstrap::init().context("Failed to initialize secrets")?;
    init_config().context("Failed to initialize configuration")?;

    let ctx = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );

    let service_id = env::var("MCP_SERVICE_ID").map_or_else(
        |_| {
            tracing::warn!(
                default = DEFAULT_SERVICE_ID,
                "MCP_SERVICE_ID not set, using default"
            );
            McpServerId::new(DEFAULT_SERVICE_ID)
        },
        McpServerId::new,
    );

    let port = env::var("MCP_PORT").map_or_else(
        |_| {
            tracing::warn!(default = DEFAULT_PORT, "MCP_PORT not set, using default");
            DEFAULT_PORT
        },
        |p| {
            p.parse::<u16>().unwrap_or_else(|e| {
                tracing::warn!(error = %e, port = %p, default = DEFAULT_PORT, "Invalid MCP_PORT, using default");
                DEFAULT_PORT
            })
        },
    );

    // Why: StubRetriever is the deliberate placeholder — swap it for the
    // Bedrock retriever once AWS credentials are provisioned (REQ-047).
    let server = KnowledgeBankServer::new(
        Arc::clone(ctx.db_pool()),
        service_id.clone(),
        Arc::clone(ctx.authz_hook()),
        Arc::new(StubRetriever),
    )
    .context("Failed to initialize KnowledgeBankServer")?;
    let router = systemprompt::mcp::create_router(
        server,
        Arc::clone(ctx.mcp_session_repository()),
        systemprompt::mcp::McpHttpConfig::default(),
    );
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!(
        service_id = %service_id,
        addr = %addr,
        "Knowledge-bank MCP server listening"
    );

    axum::serve(listener, router).await?;

    Ok(())
}
