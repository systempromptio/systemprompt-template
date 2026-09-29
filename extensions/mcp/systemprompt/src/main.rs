//! Entry point for the `systemprompt` MCP server binary.

use anyhow::{Context, Result};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use systemprompt::config::{ProfileBootstrap, SecretsBootstrap, try_init_config};
use systemprompt::identifiers::McpServerId;
use systemprompt::system::AppContext;
use systemprompt_mcp_agent::SystempromptServer;
use tokio::net::TcpListener;

const DEFAULT_SERVICE_ID: &str = "systemprompt";
const DEFAULT_PORT: u16 = 5010;
const DEFAULT_STARTUP_TIMEOUT_SECS: u64 = 60;

fn port() -> u16 {
    env::var("MCP_PORT").map_or_else(
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
    )
}

fn startup_timeout() -> Duration {
    let secs = env::var("MCP_STARTUP_TIMEOUT_SECS").map_or(DEFAULT_STARTUP_TIMEOUT_SECS, |s| {
        s.parse::<u64>().unwrap_or_else(|e| {
            tracing::warn!(error = %e, value = %s, default = DEFAULT_STARTUP_TIMEOUT_SECS, "Invalid MCP_STARTUP_TIMEOUT_SECS, using default");
            DEFAULT_STARTUP_TIMEOUT_SECS
        })
    });
    Duration::from_secs(secs)
}

async fn init_context() -> Result<Arc<AppContext>> {
    ProfileBootstrap::init().context("Failed to initialize profile")?;
    SecretsBootstrap::init()
        .await
        .context("Failed to initialize secrets")?;
    try_init_config(None).context("Failed to initialize configuration")?;
    Ok(Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    systemprompt::logging::init_console_logging();

    let addr = format!("0.0.0.0:{}", port());
    // Why: the supervisor judges this process by pid liveness and its port. The
    // port is claimed before the slow profile/secrets/database bring-up so a
    // clash fails at once, and the bring-up itself is bounded so a hang exits
    // non-zero instead of lingering alive with nothing listening.
    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind {addr}"))?;
    let timeout = startup_timeout();
    let ctx = tokio::time::timeout(timeout, init_context())
        .await
        .with_context(|| {
            format!(
                "Startup did not complete within {}s (MCP_STARTUP_TIMEOUT_SECS)",
                timeout.as_secs()
            )
        })??;

    let service_id = env::var("MCP_SERVICE_ID")
        .map_or_else(
            |_| {
                tracing::warn!(
                    default = DEFAULT_SERVICE_ID,
                    "MCP_SERVICE_ID not set, using default"
                );
                McpServerId::try_new(DEFAULT_SERVICE_ID)
            },
            McpServerId::try_new,
        )
        .context("Invalid MCP_SERVICE_ID")?;

    let server = SystempromptServer::new(
        Arc::clone(ctx.db_pool()),
        service_id.clone(),
        Arc::clone(ctx.authz_hook()),
        ctx.artifact_ingest_arc(),
    )
    .context("Failed to initialize SystempromptServer")?;
    let router = systemprompt::mcp::create_router(
        server,
        Arc::clone(ctx.mcp_session_repository()),
        systemprompt::mcp::McpHttpConfig {
            server_id: Some(service_id.clone()),
            ..systemprompt::mcp::McpHttpConfig::default()
        },
    );

    tracing::info!(
        service_id = %service_id,
        addr = %addr,
        "SystemPrompt MCP server listening"
    );

    axum::serve(listener, router).await?;

    Ok(())
}
