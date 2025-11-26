use super::{shutdown, startup, LifecycleManager};
use crate::McpServerConfig;
use anyhow::Result;
use systemprompt_core_logging::CliService;

pub async fn restart_server(manager: &LifecycleManager, config: &McpServerConfig) -> Result<()> {
    CliService::info(&format!("🔄 Restarting service: {}", config.name));

    // 1. Perform graceful stop
    CliService::info("🛑 Stopping current instance...");
    shutdown::stop_server(manager, config).await?;

    // 3. Wait for clean shutdown
    CliService::info("⏳ Waiting for clean shutdown...");
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // 4. Verify clean state
    verify_clean_state(manager, config).await?;

    // 5. Start new instance
    CliService::info("🚀 Starting new instance...");
    startup::start_server(manager, config).await?;

    CliService::success(&format!(
        "🎉 Service {} restarted successfully",
        config.name
    ));
    Ok(())
}

async fn verify_clean_state(manager: &LifecycleManager, config: &McpServerConfig) -> Result<()> {
    CliService::info("🔍 Verifying clean state...");

    // Check no process is running on the port
    if let Some(pid) = manager.process().find_pid_by_port(config.port).await? {
        return Err(anyhow::anyhow!(
            "Port {} still occupied by PID {}",
            config.port,
            pid
        ));
    }

    // Check database state is clean
    if let Some(service) = manager.database().get_service_by_name(&config.name).await? {
        if service.status == "running" {
            CliService::warning("⚠️ Database shows service as running, cleaning up...");
            manager
                .database()
                .update_service_status(&config.name, "stopped")
                .await?;
            manager.database().clear_service_pid(&config.name).await?;
        }
    }

    CliService::success("✅ Clean state verified");
    Ok(())
}
