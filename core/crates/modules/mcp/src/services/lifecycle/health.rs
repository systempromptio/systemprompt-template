use super::LifecycleManager;
use crate::services::monitoring::health::{perform_health_check, HealthStatus};
use crate::McpServerConfig;
use anyhow::Result;
use systemprompt_core_logging::CliService;

pub async fn check_server_health(
    manager: &LifecycleManager,
    config: &McpServerConfig,
) -> Result<bool> {
    let Some(pid) = manager.process().find_pid_by_port(config.port).await? else {
        manager
            .database()
            .update_service_status(&config.name, "stopped")
            .await?;
        return Ok(false);
    };

    if !manager.process().is_running(pid).await? {
        manager
            .database()
            .update_service_status(&config.name, "stopped")
            .await?;
        return Ok(false);
    }

    let health_result = perform_health_check(config).await?;

    if health_result.status != HealthStatus::Healthy
        && health_result.status != HealthStatus::Degraded
    {
        manager
            .database()
            .update_service_status(&config.name, "error")
            .await?;

        if let Some(ref error) = health_result.details.error_message {
            CliService::warning(&format!(
                "⚠️  Service {} health check: {} - {}",
                config.name,
                health_result.status.as_str(),
                error
            ));
        }
    }

    let is_healthy = matches!(
        health_result.status,
        HealthStatus::Healthy | HealthStatus::Degraded
    );

    if is_healthy && health_result.details.tools_available > 0 {
        CliService::info(&format!(
            "{} Service {} validated: {} tools available ({}ms)",
            health_result.status.emoji(),
            config.name,
            health_result.details.tools_available,
            health_result.latency_ms
        ));
    }

    Ok(is_healthy)
}
