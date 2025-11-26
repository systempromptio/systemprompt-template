use super::LifecycleManager;
use crate::services::monitoring::health::{perform_health_check, HealthStatus};
use crate::McpServerConfig;
use anyhow::Result;
use systemprompt_core_logging::CliService;

pub async fn start_server(manager: &LifecycleManager, config: &McpServerConfig) -> Result<()> {
    CliService::info(&format!(
        "🚀 Starting server: {} on port {} (OAuth: {})",
        config.name, config.port, config.oauth.required
    ));

    // 1. Verify prerequisites
    verify_prerequisites(manager, config).await?;

    // 2. Prepare network resources
    manager.network().prepare_port(config.port).await?;

    // 3. Start the process
    let pid = manager.process().spawn_server(config).await?;

    // 4. Wait for startup and validate
    let startup_time = wait_for_startup(manager, config, pid).await?;

    // 5. Register in database
    let service_id = manager
        .database()
        .register_service(config, pid, startup_time)
        .await?;

    // 6. Initialize monitoring
    manager
        .monitoring()
        .start_monitoring(config, service_id.clone())
        .await?;

    CliService::success(&format!("✅ Server {} started successfully", config.name));
    CliService::info(&format!("   📋 PID: {pid}"));
    CliService::info(&format!("   🗃️  Service ID: {service_id}"));
    CliService::info(&format!("   🌐 Port: {}", config.port));
    CliService::info(&format!(
        "   🔗 URL: http://{}:{}",
        config.host, config.port
    ));
    if let Some(startup_time) = startup_time {
        CliService::info(&format!("   ⏱️  Startup Time: {startup_time}ms"));
    }

    Ok(())
}

async fn verify_prerequisites(manager: &LifecycleManager, config: &McpServerConfig) -> Result<()> {
    CliService::info("🔍 Verifying prerequisites...");

    // Check if already running
    if let Some(pid) = manager.process().find_pid_by_port(config.port).await? {
        return Err(anyhow::anyhow!(
            "Service already running on port {} (PID: {})",
            config.port,
            pid
        ));
    }

    // Verify binary exists
    manager.process().verify_binary(config).await?;

    CliService::success("✅ Prerequisites verified");
    Ok(())
}

async fn wait_for_startup(
    manager: &LifecycleManager,
    config: &McpServerConfig,
    expected_pid: u32,
) -> Result<Option<i32>> {
    CliService::info("⏳ Waiting for service to become available...");

    let start_time = std::time::Instant::now();
    let max_attempts = 15;
    let base_delay = std::time::Duration::from_millis(300);

    for attempt in 1..=max_attempts {
        let delay = if attempt == 1 {
            std::time::Duration::from_millis(500)
        } else {
            base_delay * std::cmp::min(attempt, 5)
        };
        tokio::time::sleep(delay).await;

        CliService::info(&format!(
            "   Attempt {attempt}/{max_attempts}: Checking service health..."
        ));

        if !manager.process().is_running(expected_pid).await? {
            return Err(anyhow::anyhow!(
                "Process {expected_pid} died during startup"
            ));
        }

        if !manager.network().is_port_responsive(config.port).await? {
            continue;
        }

        match perform_health_check(config).await {
            Ok(health_result) => {
                let startup_time_ms = start_time.elapsed().as_millis() as i32;

                match health_result.status {
                    HealthStatus::Healthy => {
                        if health_result.details.requires_auth {
                            CliService::success(&format!(
                                "✅ Service responding (OAuth required) - startup: {startup_time_ms}ms"
                            ));
                        } else {
                            CliService::success(&format!(
                                "✅ MCP service validated: {} tools available (startup: {}ms)",
                                health_result.details.tools_available, startup_time_ms
                            ));
                        }
                        return Ok(Some(startup_time_ms));
                    },
                    HealthStatus::Degraded => {
                        if attempt >= max_attempts - 2 {
                            let error_msg = health_result
                                .details
                                .error_message
                                .as_deref()
                                .filter(|e| !e.is_empty())
                                .unwrap_or("[degraded - no error message]");
                            CliService::warning(&format!(
                                "⚠️  Service degraded but accepting: {error_msg} (startup: {startup_time_ms}ms)"
                            ));
                            return Ok(Some(startup_time_ms));
                        }
                    },
                    _ => {
                        if let Some(ref err_msg) = health_result.details.error_message {
                            CliService::info(&format!("   Health check: {err_msg}"));
                        }
                    },
                }
            },
            Err(e) => {
                if attempt >= max_attempts - 5 {
                    CliService::info(&format!("   Health check error: {e}"));
                }
            },
        }
    }

    Err(anyhow::anyhow!(
        "Service {} failed health validation after {} attempts",
        config.name,
        max_attempts
    ))
}
