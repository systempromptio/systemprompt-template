use super::LifecycleManager;
use crate::McpServerConfig;
use anyhow::Result;
use systemprompt_core_logging::CliService;

pub async fn stop_server(manager: &LifecycleManager, config: &McpServerConfig) -> Result<()> {
    CliService::info(&format!("🛑 Stopping service: {}", config.name));

    // 1. Find the running process
    let Some(pid) = find_running_process(manager, config).await? else {
        CliService::info(&format!("✅ Service {} is already stopped", config.name));
        cleanup_stale_state(manager, config).await?;
        return Ok(());
    };

    // 2. Update status to stopping
    manager
        .database()
        .update_service_status(&config.name, "stopping")
        .await?;

    // 3. Stop monitoring
    manager.monitoring().stop_monitoring(&config.name).await?;

    // 4. Graceful shutdown
    perform_graceful_shutdown(manager, config, pid).await?;

    // 5. Update database state
    finalize_shutdown(manager, config).await?;

    CliService::success(&format!("✅ Service {} stopped successfully", config.name));
    Ok(())
}

async fn find_running_process(
    manager: &LifecycleManager,
    config: &McpServerConfig,
) -> Result<Option<u32>> {
    // First check database
    if let Some(db_service) = manager.database().get_service_by_name(&config.name).await? {
        if let Some(db_pid) = db_service.pid {
            if manager.process().is_running(db_pid as u32).await? {
                return Ok(Some(db_pid as u32));
            }
        }
    }

    // Fallback to port-based detection
    manager.process().find_pid_by_port(config.port).await
}

async fn perform_graceful_shutdown(
    manager: &LifecycleManager,
    config: &McpServerConfig,
    pid: u32,
) -> Result<()> {
    CliService::info(&format!("🔄 Performing graceful shutdown for PID {pid}"));

    // Send SIGTERM for graceful shutdown
    manager.process().terminate_gracefully(pid).await?;

    // Wait for graceful shutdown
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Force kill if still running
    if manager.process().is_running(pid).await? {
        CliService::info(&format!("⚡ Force killing PID {pid}"));
        manager.process().force_kill(pid).await?;
    }

    // Wait for port to be released
    manager.network().wait_for_port_release(config.port).await?;

    Ok(())
}

async fn finalize_shutdown(manager: &LifecycleManager, config: &McpServerConfig) -> Result<()> {
    // Update database status
    manager
        .database()
        .update_service_status(&config.name, "stopped")
        .await?;
    manager.database().clear_service_pid(&config.name).await?;

    // Clean up any remaining resources
    manager
        .network()
        .cleanup_port_resources(config.port)
        .await?;

    Ok(())
}

async fn cleanup_stale_state(manager: &LifecycleManager, config: &McpServerConfig) -> Result<()> {
    CliService::info("🧹 Cleaning up stale database entries...");

    if let Some(service) = manager.database().get_service_by_name(&config.name).await? {
        manager.database().unregister_service(&service.name).await?;
        CliService::success(&format!("✅ Cleaned up stale entry for {}", config.name));
    }

    Ok(())
}
