use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_logging::{CliService, LogLevel, LogService};

use super::event_bus::EventBus;
use super::events::McpEvent;
use crate::services::database::DatabaseManager;
use crate::services::lifecycle::LifecycleManager;
use crate::services::registry::RegistryManager;

pub async fn run_daemon(
    event_bus: &Arc<EventBus>,
    lifecycle: &LifecycleManager,
    registry: &RegistryManager,
    database: &DatabaseManager,
    logger: &LogService,
) -> Result<()> {
    CliService::info("Starting MCP daemon mode...");

    database.cleanup_stale_services().await?;
    let servers = registry.get_enabled_servers().await?;
    database.sync_state(&servers).await?;
    let server_count = servers.len();

    logger
        .log(
            LogLevel::Info,
            "mcp_orchestrator",
            "MCP daemon started",
            Some(serde_json::json!({
                "mode": "daemon",
                "enabled_services": server_count,
                "services": servers.iter().map(|s| s.name.clone()).collect::<Vec<_>>()
            })),
        )
        .await
        .ok();

    for server in &servers {
        event_bus
            .publish(McpEvent::ServiceStartRequested {
                service_name: server.name.clone(),
            })
            .await?;
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    CliService::success("All MCP servers started with proper OAuth enforcement");
    CliService::info("MCP manager will keep servers running. Press Ctrl+C to stop.");

    tokio::signal::ctrl_c().await?;
    CliService::info("Shutting down MCP servers...");

    let running_servers = database.get_running_servers().await?;
    let running_count = running_servers.len();

    logger
        .log(
            LogLevel::Info,
            "mcp_orchestrator",
            "MCP daemon shutdown initiated",
            Some(serde_json::json!({
                "running_services": running_count,
                "shutdown_reason": "user_interrupt"
            })),
        )
        .await
        .ok();

    for server in running_servers {
        lifecycle.stop_server(&server).await?;

        event_bus
            .publish(McpEvent::ServiceStopped {
                service_name: server.name,
                exit_code: None,
            })
            .await?;
    }

    logger
        .info("mcp_orchestrator", "MCP daemon shutdown completed")
        .await
        .ok();

    Ok(())
}
