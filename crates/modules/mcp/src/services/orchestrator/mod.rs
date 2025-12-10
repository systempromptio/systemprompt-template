use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_logging::{CliService, LogService};
use systemprompt_core_system::AppContext;

mod daemon;
pub mod event_bus;
pub mod events;
pub mod handlers;
mod reconciliation;
mod service_validation;

pub use event_bus::EventBus;
pub use events::McpEvent;
pub use handlers::{DatabaseSyncHandler, HealthCheckHandler, LifecycleHandler, MonitoringHandler};

use super::database::DatabaseManager;
use super::lifecycle::LifecycleManager;
use super::monitoring::MonitoringManager;
use super::network::NetworkManager;
use super::process::ProcessManager;
use super::registry::RegistryManager;
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
        let lifecycle =
            LifecycleManager::new(process, network, database.clone(), monitoring.clone());
        let log = LogService::system(ctx.db_pool().clone());

        event_bus.register_handler(Arc::new(LifecycleHandler::new(lifecycle.clone(), registry)));

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
            CliService::info(&format!("Building service: {}", server.name));
            self.lifecycle.process().build_server(&server).await?;
        }

        CliService::success("Build completed");
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
        reconciliation::reconcile(
            &self.database,
            &self.registry,
            &self.lifecycle,
            &self.event_bus,
            &self.app_context,
            &self.log,
        )
        .await
    }

    pub async fn validate_service(&self, service_name: &str) -> Result<()> {
        service_validation::validate_service(service_name, &self.registry, &self.database).await
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
        daemon::run_daemon(
            &self.event_bus,
            &self.lifecycle,
            &self.registry,
            &self.database,
            &self.log,
        )
        .await
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
