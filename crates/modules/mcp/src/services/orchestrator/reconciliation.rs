use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_logging::{CliService, LogLevel, LogService};
use systemprompt_core_system::AppContext;

use super::event_bus::EventBus;
use super::events::McpEvent;
use crate::services::database::DatabaseManager;
use crate::services::lifecycle::LifecycleManager;
use crate::services::registry::RegistryManager;
use crate::services::schema::{SchemaValidationMode, SchemaValidationReport, SchemaValidator};
use crate::McpServerConfig;

pub async fn reconcile(
    database: &DatabaseManager,
    registry: &RegistryManager,
    lifecycle: &LifecycleManager,
    event_bus: &Arc<EventBus>,
    app_context: &Arc<AppContext>,
    logger: &LogService,
) -> Result<usize> {
    database.cleanup_stale_services().await?;
    database.delete_crashed_services().await?;

    let enabled_servers = registry.get_enabled_servers().await?;

    let schema_report = validate_and_migrate_schemas(&enabled_servers, app_context, logger).await?;
    if !schema_report.errors.is_empty() {
        for error in &schema_report.errors {
            CliService::error(&format!("Schema error: {error}"));
        }
        return Err(anyhow::anyhow!(
            "Schema validation failed with {} errors",
            schema_report.errors.len()
        ));
    }

    if schema_report.created > 0 {
        CliService::success(&format!("Created {} missing tables", schema_report.created));
    }

    database.sync_state(&enabled_servers).await?;

    let orphaned =
        detect_and_handle_orphaned_processes(&enabled_servers, lifecycle, database, logger).await?;
    if orphaned > 0 {
        CliService::success(&format!(
            "Killed {orphaned} orphaned MCP processes, will restart fresh"
        ));
    }

    let stale =
        detect_and_handle_stale_binaries(&enabled_servers, lifecycle, database, logger).await?;
    if stale > 0 {
        CliService::success(&format!(
            "Killed {stale} stale MCP processes (binary rebuilt), will restart fresh"
        ));
    }

    let running_servers = database.get_running_servers().await?;
    let running_names: std::collections::HashSet<String> =
        running_servers.iter().map(|s| s.name.clone()).collect();

    let mut failed: Vec<(String, String)> = Vec::new();

    for server in enabled_servers {
        if running_names.contains(&server.name) {
            continue;
        }

        event_bus
            .publish(McpEvent::ServiceStartRequested {
                service_name: server.name.clone(),
            })
            .await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        if let Some(service_info) = database.get_service_by_name(&server.name).await? {
            if service_info.status == "running" {
                event_bus
                    .publish(McpEvent::ServiceStarted {
                        service_name: server.name.clone(),
                        process_id: service_info.pid.unwrap_or(0) as u32,
                        port: server.port,
                    })
                    .await?;
            } else {
                let error_msg = format!("Failed to start {}", server.name);
                failed.push((server.name.clone(), error_msg.clone()));
                event_bus
                    .publish(McpEvent::ServiceFailed {
                        service_name: server.name,
                        error: error_msg,
                    })
                    .await?;
            }
        }
    }

    if !failed.is_empty() {
        return Err(anyhow::anyhow!(
            "Failed to start {} MCP service(s): {}",
            failed.len(),
            failed
                .iter()
                .map(|(name, err)| format!("{name} ({err})"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let running = database.get_running_servers().await?;
    Ok(running.len())
}

pub async fn validate_and_migrate_schemas(
    servers: &[McpServerConfig],
    app_context: &Arc<AppContext>,
    logger: &LogService,
) -> Result<SchemaValidationReport> {
    use systemprompt_core_config::services::ConfigLoader;

    let services_config = ConfigLoader::load().await?;
    let validation_mode =
        SchemaValidationMode::from_string(&services_config.settings.schema_validation_mode);

    let validator = SchemaValidator::new(app_context.db_pool().as_ref(), validation_mode);

    let mut combined_report = SchemaValidationReport::new("all".to_string());

    for server in servers {
        if server.schemas.is_empty() {
            continue;
        }

        let service_path = std::path::Path::new(&server.crate_path);

        match validator
            .validate_and_apply(&server.name, service_path, &server.schemas)
            .await
        {
            Ok(report) => {
                if report.validated > 0 {
                    logger
                        .log(
                            LogLevel::Info,
                            "mcp_orchestrator",
                            &format!("Validated schemas for MCP service: {}", server.name),
                            Some(serde_json::json!({
                                "service_name": server.name,
                                "validated": report.validated,
                                "created": report.created,
                            })),
                        )
                        .await
                        .ok();
                }
                combined_report.merge(report);
            },
            Err(e) => {
                let error_msg = format!("Schema validation failed for {}: {}", server.name, e);
                combined_report.errors.push(error_msg.clone());

                logger
                    .log(
                        LogLevel::Error,
                        "mcp_orchestrator",
                        &error_msg,
                        Some(serde_json::json!({
                            "service_name": server.name,
                            "failure_reason": e.to_string(),
                        })),
                    )
                    .await
                    .ok();
            },
        }
    }

    Ok(combined_report)
}

pub async fn detect_and_handle_orphaned_processes(
    servers: &[McpServerConfig],
    lifecycle: &LifecycleManager,
    database: &DatabaseManager,
    logger: &LogService,
) -> Result<usize> {
    let mut killed = 0;

    for server in servers {
        if let Some(orphaned_pid) = lifecycle
            .process()
            .find_process_on_port_with_name(server.port, &server.name)
            .await?
        {
            if database.get_service_by_name(&server.name).await?.is_none() {
                CliService::info(&format!(
                    "Found orphaned process: {} (PID {}) on port {}",
                    server.name, orphaned_pid, server.port
                ));

                lifecycle.process().force_kill(orphaned_pid).await?;
                killed += 1;

                logger
                    .log(
                        LogLevel::Info,
                        "mcp_orchestrator",
                        &format!(
                            "Killed orphaned MCP process, will restart fresh: {}",
                            server.name
                        ),
                        Some(serde_json::json!({
                            "service_name": server.name,
                            "pid": orphaned_pid,
                            "port": server.port,
                        })),
                    )
                    .await
                    .ok();

                CliService::success(&format!(
                    "Killed orphaned process {} (PID: {}), will restart fresh",
                    server.name, orphaned_pid
                ));
            }
        }
    }

    Ok(killed)
}

pub async fn detect_and_handle_stale_binaries(
    servers: &[McpServerConfig],
    lifecycle: &LifecycleManager,
    database: &DatabaseManager,
    logger: &LogService,
) -> Result<usize> {
    use crate::services::database::state::get_binary_mtime_for_service;

    let mut restarted = 0;

    for server in servers {
        let Some(service_info) = database.get_service_by_name(&server.name).await? else {
            continue;
        };

        if service_info.status != "running" {
            continue;
        }

        let Some(stored_mtime) = service_info.binary_mtime else {
            continue;
        };

        let current_mtime = get_binary_mtime_for_service(&server.name);
        let Some(current_mtime) = current_mtime else {
            continue;
        };

        if current_mtime != stored_mtime {
            CliService::info(&format!(
                "🔄 Binary rebuilt for '{}': stored={}, current={} - restarting",
                server.name, stored_mtime, current_mtime
            ));

            if let Some(pid) = service_info.pid {
                lifecycle.process().force_kill(pid as u32).await?;
            }

            database.unregister_service(&server.name).await?;
            restarted += 1;

            logger
                .log(
                    LogLevel::Info,
                    "mcp_orchestrator",
                    &format!(
                        "Killed stale binary process, will restart with new binary: {}",
                        server.name
                    ),
                    Some(serde_json::json!({
                        "service_name": server.name,
                        "pid": service_info.pid,
                        "stored_mtime": stored_mtime,
                        "current_mtime": current_mtime,
                    })),
                )
                .await
                .ok();

            CliService::success(&format!(
                "✅ Killed stale process {} (PID: {:?}), will restart with new binary",
                server.name, service_info.pid
            ));
        }
    }

    Ok(restarted)
}
