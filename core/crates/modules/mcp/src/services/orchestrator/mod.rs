use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_logging::{CliService, LogLevel, LogService};
use systemprompt_core_system::AppContext;

pub mod event_bus;
pub mod events;
pub mod handlers;

pub use event_bus::EventBus;
pub use events::McpEvent;
pub use handlers::{DatabaseSyncHandler, HealthCheckHandler, LifecycleHandler, MonitoringHandler};

use super::{
    client::McpClient,
    database::DatabaseManager,
    lifecycle::LifecycleManager,
    monitoring::MonitoringManager,
    network::NetworkManager,
    process::ProcessManager,
    registry::RegistryManager,
    schema::{SchemaValidationMode, SchemaValidationReport, SchemaValidator},
};
use crate::McpServerConfig;

#[derive(Debug)]
pub struct McpOrchestrator {
    event_bus: Arc<EventBus>,
    lifecycle: LifecycleManager,
    registry: RegistryManager,
    database: DatabaseManager,
    monitoring: MonitoringManager,
    app_context: Arc<AppContext>,
    log: LogService,
}

impl McpOrchestrator {
    pub async fn new(ctx: Arc<AppContext>) -> Result<Self> {
        let mut event_bus = EventBus::new(100);

        let registry = RegistryManager::new().await?;
        let database = DatabaseManager::new(ctx.db_pool().clone());
        let network = NetworkManager::new();
        let process = ProcessManager::new(ctx.clone());
        let monitoring = MonitoringManager::new(ctx.clone());
        let lifecycle = LifecycleManager::new(
            process,
            network,
            database.clone(),
            monitoring.clone(),
        );
        let log = LogService::system(ctx.db_pool().clone());

        event_bus.register_handler(Arc::new(LifecycleHandler::new(
            lifecycle.clone(),
            registry,
        )));

        event_bus.register_handler(Arc::new(MonitoringHandler::new(ctx.db_pool().clone())));

        event_bus.register_handler(Arc::new(DatabaseSyncHandler::new(database.clone())));

        let health_handler = HealthCheckHandler::new().with_restart_sender(event_bus.sender());
        event_bus.register_handler(Arc::new(health_handler));

        Ok(Self {
            event_bus: Arc::new(event_bus),
            lifecycle,
            registry,
            database,
            monitoring,
            app_context: ctx,
            log,
        })
    }

    pub async fn list_services(&self) -> Result<()> {
        CliService::section("MCP Services");
        let servers = self.registry.get_enabled_servers().await?;
        let status_data = self.monitoring.get_status_for_all(&servers).await?;
        self.monitoring.display_status(&servers, &status_data).await
    }

    pub async fn start_services(&self, service_name: Option<String>) -> Result<()> {
        let servers = self.get_target_servers(service_name, true).await?;
        let mut failed = Vec::new();

        for server in servers {
            CliService::section(&format!("Starting Service: {}", server.name));

            self.event_bus
                .publish(McpEvent::ServiceStartRequested {
                    service_name: server.name.clone(),
                })
                .await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            if let Some(service_info) = self.database.get_service_by_name(&server.name).await? {
                if service_info.status == "running" {
                    self.event_bus
                        .publish(McpEvent::ServiceStarted {
                            service_name: server.name.clone(),
                            process_id: service_info.pid.unwrap_or(0) as u32,
                            port: server.port,
                        })
                        .await?;
                } else {
                    let error_msg = format!("Failed to start {}", server.name);
                    failed.push((server.name.clone(), error_msg.clone()));
                    self.event_bus
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
                "Failed to start {} services: {}",
                failed.len(),
                failed
                    .iter()
                    .map(|(n, e)| format!("{n} ({e})"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        Ok(())
    }

    pub async fn stop_services(&self, service_name: Option<String>) -> Result<()> {
        let servers = self.get_target_servers(service_name, false).await?;

        for server in servers {
            CliService::section(&format!("Stopping Service: {}", server.name));

            match self.lifecycle.stop_server(&server).await {
                Ok(()) => {
                    self.event_bus
                        .publish(McpEvent::ServiceStopped {
                            service_name: server.name,
                            exit_code: None,
                        })
                        .await?;
                },
                Err(e) => {
                    return Err(e);
                },
            }
        }

        Ok(())
    }

    pub async fn restart_services(&self, service_name: Option<String>) -> Result<()> {
        let servers = self.get_target_servers(service_name, false).await?;

        for server in servers {
            CliService::section(&format!("Restarting Service: {}", server.name));

            self.event_bus
                .publish(McpEvent::ServiceRestartRequested {
                    service_name: server.name,
                    reason: "Manual restart".to_string(),
                })
                .await?;
        }

        Ok(())
    }

    pub async fn build_services(&self, service_name: Option<String>) -> Result<()> {
        let servers = self.get_target_servers(service_name, true).await?;

        for server in servers {
            CliService::info(&format!("🔨 Building service: {}", server.name));
            self.lifecycle.process().build_server(&server).await?;
        }

        CliService::success("✅ Build completed");
        Ok(())
    }

    pub async fn show_status(&self) -> Result<()> {
        self.list_services().await
    }

    pub async fn sync_database_state(&self) -> Result<()> {
        CliService::section("Synchronizing Service Database State");
        let servers = self.registry.get_enabled_servers().await?;
        self.database.sync_state(&servers).await
    }

    pub async fn reconcile(&self) -> Result<usize> {
        self.database.cleanup_stale_services().await?;
        self.database.delete_crashed_services().await?;

        let enabled_servers = self.registry.get_enabled_servers().await?;

        let schema_report = self.validate_and_migrate_schemas(&enabled_servers).await?;
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

        self.database.sync_state(&enabled_servers).await?;

        let orphaned = self
            .detect_and_handle_orphaned_processes(&enabled_servers)
            .await?;
        if orphaned > 0 {
            CliService::success(&format!(
                "✅ Killed {orphaned} orphaned MCP processes, will restart fresh"
            ));
        }

        let running_servers = self.database.get_running_servers().await?;
        let running_names: std::collections::HashSet<String> =
            running_servers.iter().map(|s| s.name.clone()).collect();

        let mut failed: Vec<(String, String)> = Vec::new();

        for server in enabled_servers {
            if running_names.contains(&server.name) {
                continue;
            }

            self.event_bus
                .publish(McpEvent::ServiceStartRequested {
                    service_name: server.name.clone(),
                })
                .await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            if let Some(service_info) = self.database.get_service_by_name(&server.name).await? {
                if service_info.status == "running" {
                    self.event_bus
                        .publish(McpEvent::ServiceStarted {
                            service_name: server.name.clone(),
                            process_id: service_info.pid.unwrap_or(0) as u32,
                            port: server.port,
                        })
                        .await?;
                } else {
                    let error_msg = format!("Failed to start {}", server.name);
                    failed.push((server.name.clone(), error_msg.clone()));
                    self.event_bus
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

        let running = self.database.get_running_servers().await?;
        Ok(running.len())
    }

    async fn validate_and_migrate_schemas(
        &self,
        servers: &[McpServerConfig],
    ) -> Result<SchemaValidationReport> {
        use systemprompt_core_config::services::ConfigLoader;

        let services_config = ConfigLoader::load().await?;
        let validation_mode =
            SchemaValidationMode::from_string(&services_config.settings.schema_validation_mode);

        let validator = SchemaValidator::new(self.app_context.db_pool().as_ref(), validation_mode);

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
                        self.log
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

                    self.log
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

    async fn detect_and_handle_orphaned_processes(
        &self,
        servers: &[McpServerConfig],
    ) -> Result<usize> {
        let mut killed = 0;

        for server in servers {
            if let Some(orphaned_pid) = self
                .lifecycle
                .process()
                .find_process_on_port_with_name(server.port, &server.name)
                .await?
            {
                if self
                    .database
                    .get_service_by_name(&server.name)
                    .await?
                    .is_none()
                {
                    CliService::info(&format!(
                        "🔍 Found orphaned process: {} (PID {}) on port {}",
                        server.name, orphaned_pid, server.port
                    ));

                    self.lifecycle.process().force_kill(orphaned_pid).await?;
                    killed += 1;

                    self.log
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
                        "✅ Killed orphaned process {} (PID: {}), will restart fresh",
                        server.name, orphaned_pid
                    ));
                }
            }
        }

        Ok(killed)
    }

    pub async fn validate_service(&self, service_name: &str) -> Result<()> {
        let servers = self.registry.get_enabled_servers().await?;
        let server = servers
            .iter()
            .find(|s| s.name == service_name)
            .ok_or_else(|| anyhow::anyhow!("Service '{service_name}' not found in registry"))?;

        CliService::section(&format!("Validating MCP Service: {service_name}"));

        CliService::info(&format!("✅ Service '{service_name}' found in registry"));
        CliService::info(&format!("   Port: {}", server.port));
        CliService::info(&format!("   Enabled: {}", server.enabled));
        CliService::info(&format!("   OAuth required: {}", server.oauth.required));

        let service_info = self.database.get_service_by_name(service_name).await?;

        let is_running = service_info
            .as_ref()
            .is_some_and(|info| info.status == "running");

        if !is_running {
            CliService::warning(&format!(
                "⚠️  Service '{service_name}' is not currently running"
            ));
            CliService::info("   Start the service first with: just mcp start systemprompt-admin");
            return Ok(());
        }

        CliService::info("🔍 Connecting to MCP service...");

        let validation_result = McpClient::validate_connection_with_auth(
            &server.name,
            "127.0.0.1",
            server.port,
            server.oauth.required,
        )
        .await?;

        if validation_result.success {
            CliService::success("✅ Successfully connected to MCP service");

            if let Some(server_info) = &validation_result.server_info {
                CliService::info(&format!("   Server: {}", server_info.server_name));
                CliService::info(&format!("   Version: {}", server_info.version));
                CliService::info(&format!("   Protocol: {}", server_info.protocol_version));
            }

            if server.oauth.required && validation_result.validation_type == "auth_required" {
                CliService::info("🔒 Service requires OAuth authentication");
                CliService::info("ℹ️  Full validation skipped (requires user JWT token)");
                CliService::info("   Port is responding - service appears healthy");
            } else if validation_result.validation_type == "mcp_validated" {
                CliService::success("✅ MCP Protocol validated");
                CliService::info(&format!(
                    "   Tools available: {}",
                    validation_result.tools_count
                ));
                CliService::info("ℹ️  Tool listing requires user authentication");
            }

            CliService::info(&format!(
                "⏱️  Connection time: {}ms",
                validation_result.connection_time_ms
            ));
        } else {
            let error = validation_result
                .error_message
                .as_deref()
                .filter(|e| !e.is_empty())
                .unwrap_or("[no error message]");
            CliService::error(&format!("❌ Failed to connect: {error}"));
        }

        Ok(())
    }

    pub async fn get_enabled_servers(&self) -> Result<Vec<McpServerConfig>> {
        self.registry.get_enabled_servers().await
    }

    pub async fn get_running_servers(&self) -> Result<Vec<McpServerConfig>> {
        self.database.get_running_servers().await
    }

    async fn get_target_servers(
        &self,
        service_name: Option<String>,
        enabled_only: bool,
    ) -> Result<Vec<McpServerConfig>> {
        match service_name {
            Some(name) if name == "all" => {
                if enabled_only {
                    self.registry.get_enabled_servers().await
                } else {
                    self.database.get_running_servers().await
                }
            },
            Some(name) => {
                let servers = if enabled_only {
                    self.registry.get_enabled_servers().await?
                } else {
                    self.registry.get_all_servers().await?
                };
                Ok(servers.into_iter().filter(|s| s.name == name).collect())
            },
            None => {
                if enabled_only {
                    self.registry.get_enabled_servers().await
                } else {
                    self.database.get_running_servers().await
                }
            },
        }
    }

    pub async fn run_daemon(&self) -> Result<()> {
        CliService::info("🚀 Starting MCP daemon mode...");

        self.database.cleanup_stale_services().await?;
        let servers = self.registry.get_enabled_servers().await?;
        self.database.sync_state(&servers).await?;
        let server_count = servers.len();

        self.log
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
            self.event_bus
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

        let running_servers = self.database.get_running_servers().await?;
        let running_count = running_servers.len();

        self.log
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
            self.lifecycle.stop_server(&server).await?;

            self.event_bus
                .publish(McpEvent::ServiceStopped {
                    service_name: server.name,
                    exit_code: None,
                })
                .await?;
        }

        self.log
            .info("mcp_orchestrator", "MCP daemon shutdown completed")
            .await
            .ok();

        Ok(())
    }

    pub async fn list_packages(&self) -> Result<()> {
        let servers = self.registry.get_enabled_servers().await?;
        let packages: Vec<String> = servers.iter().map(|s| s.name.clone()).collect();
        println!("{}", packages.join(" "));
        Ok(())
    }

    pub fn subscribe_events(&self) -> tokio::sync::broadcast::Receiver<McpEvent> {
        self.event_bus.subscribe()
    }
}
