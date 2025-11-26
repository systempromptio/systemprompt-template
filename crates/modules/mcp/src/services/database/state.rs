use super::ServiceInfo;
use crate::McpServerConfig;
use anyhow::Result;
use systemprompt_models::repository::ServiceRepository;

pub async fn register_service(
    db_pool: &systemprompt_core_database::DbPool,
    config: &McpServerConfig,
    pid: u32,
    _startup_time: Option<i32>,
) -> Result<String> {
    use systemprompt_core_logging::CliService;

    let repo = ServiceRepository::new(db_pool.clone());

    // Register service: create with status='running' and store PID
    CliService::info(&format!(
        "📝 Registering MCP service '{}' (PID: {}, port: {})",
        config.name, pid, config.port
    ));

    repo.create_service(&config.name, "mcp", "running", config.port)
        .await
        .map_err(|e| {
            CliService::error(&format!(
                "❌ Failed to create service record for '{}': {}",
                config.name, e
            ));
            e
        })?;

    repo.update_service_pid(&config.name, pid as i32)
        .await
        .map_err(|e| {
            CliService::error(&format!(
                "❌ Failed to update PID for service '{}': {}",
                config.name, e
            ));
            e
        })?;

    CliService::success(&format!(
        "✅ Service '{}' registered in database (PID: {})",
        config.name, pid
    ));

    Ok(config.name.clone())
}

pub async fn unregister_service(
    db_pool: &systemprompt_core_database::DbPool,
    service_name: &str,
) -> Result<()> {
    let repo = ServiceRepository::new(db_pool.clone());
    repo.delete_service(service_name).await
}

pub async fn get_service_by_name(
    db_pool: &systemprompt_core_database::DbPool,
    name: &str,
) -> Result<Option<ServiceInfo>> {
    let repo = ServiceRepository::new(db_pool.clone());
    let result = repo.get_service_by_name(name).await?;

    Ok(result.map(|r| ServiceInfo {
        name: r.name,
        status: r.status,
        pid: r.pid,
        port: r.port as u16,
    }))
}

pub async fn get_running_servers(
    db_pool: &systemprompt_core_database::DbPool,
) -> Result<Vec<McpServerConfig>> {
    use crate::services::registry::RegistryManager;

    let repo = ServiceRepository::new(db_pool.clone());
    let all_services = repo.get_mcp_services().await?;

    let registry = RegistryManager::new().await?;
    let mut running_configs = Vec::new();

    for service in all_services {
        if service.status == "running" {
            if let Some(config) = registry.get_server_by_name(&service.name).await? {
                running_configs.push(config);
            }
        }
    }

    Ok(running_configs)
}

pub async fn update_service_state(
    db_pool: &systemprompt_core_database::DbPool,
    name: &str,
    status: &str,
    _pid: Option<u32>,
) -> Result<()> {
    let repo = ServiceRepository::new(db_pool.clone());
    repo.update_service_status(name, status).await
}

pub async fn register_existing_process(
    db_pool: &systemprompt_core_database::DbPool,
    config: &McpServerConfig,
    pid: u32,
) -> Result<String> {
    let repo = ServiceRepository::new(db_pool.clone());

    repo.create_service(&config.name, "mcp", "running", config.port)
        .await?;

    repo.update_service_pid(&config.name, pid as i32).await?;

    Ok(config.name.clone())
}
