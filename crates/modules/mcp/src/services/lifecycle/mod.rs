pub mod health;
pub mod restart;
pub mod shutdown;
pub mod startup;

use crate::services::database::DatabaseManager;
use crate::services::monitoring::MonitoringManager;
use crate::services::network::NetworkManager;
use crate::services::process::ProcessManager;
use crate::McpServerConfig;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct LifecycleManager {
    process: ProcessManager,
    network: NetworkManager,
    database: DatabaseManager,
    monitoring: MonitoringManager,
}

impl LifecycleManager {
    pub const fn new(
        process: ProcessManager,
        network: NetworkManager,
        database: DatabaseManager,
        monitoring: MonitoringManager,
    ) -> Self {
        Self {
            process,
            network,
            database,
            monitoring,
        }
    }

    pub async fn start_server(&self, config: &McpServerConfig) -> Result<()> {
        startup::start_server(self, config).await
    }

    pub async fn stop_server(&self, config: &McpServerConfig) -> Result<()> {
        shutdown::stop_server(self, config).await
    }

    pub async fn restart_server(&self, config: &McpServerConfig) -> Result<()> {
        restart::restart_server(self, config).await
    }

    pub async fn health_check(&self, config: &McpServerConfig) -> Result<bool> {
        health::check_server_health(self, config).await
    }

    pub const fn process(&self) -> &ProcessManager {
        &self.process
    }

    pub const fn network(&self) -> &NetworkManager {
        &self.network
    }

    pub const fn database(&self) -> &DatabaseManager {
        &self.database
    }

    pub const fn monitoring(&self) -> &MonitoringManager {
        &self.monitoring
    }
}
