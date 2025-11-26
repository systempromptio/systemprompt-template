pub mod health;
pub mod metrics;
pub mod status;

use crate::McpServerConfig;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use systemprompt_core_system::AppContext;

#[derive(Debug, Clone)]
pub struct MonitoringManager {
    app_context: Arc<AppContext>,
}

impl MonitoringManager {
    pub const fn new(app_context: Arc<AppContext>) -> Self {
        Self { app_context }
    }

    pub async fn start_monitoring(
        &self,
        _config: &McpServerConfig,
        _service_id: String,
    ) -> Result<()> {
        // Start background monitoring task
        Ok(())
    }

    pub async fn stop_monitoring(&self, _name: &str) -> Result<()> {
        // Stop monitoring task for service
        Ok(())
    }

    pub async fn check_health(&self, config: &McpServerConfig) -> Result<health::HealthStatus> {
        health::check_service_health(config).await
    }

    pub async fn get_status_for_all(
        &self,
        servers: &[McpServerConfig],
    ) -> Result<HashMap<String, status::ServiceStatus>> {
        status::get_all_service_status(servers).await
    }

    pub async fn display_status(
        &self,
        servers: &[McpServerConfig],
        status_data: &HashMap<String, status::ServiceStatus>,
    ) -> Result<()> {
        status::display_service_status(servers, status_data).await
    }

    pub async fn collect_metrics(
        &self,
        config: &McpServerConfig,
    ) -> Result<metrics::ServicePerformanceMetrics> {
        metrics::collect_service_metrics(config).await
    }

    pub const fn app_context(&self) -> &Arc<AppContext> {
        &self.app_context
    }
}
