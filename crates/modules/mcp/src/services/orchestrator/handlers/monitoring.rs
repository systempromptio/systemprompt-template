use anyhow::Result;
use async_trait::async_trait;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::{LogLevel, LogService};

use super::{EventHandler, McpEvent};

#[derive(Debug)]
pub struct MonitoringHandler {
    log: LogService,
}

impl MonitoringHandler {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            log: LogService::system(db_pool),
        }
    }
}

#[async_trait]
impl EventHandler for MonitoringHandler {
    async fn handle(&self, event: &McpEvent) -> Result<()> {
        match event {
            McpEvent::ServiceStarted {
                service_name,
                process_id,
                port,
            } => {
                tracing::info!("Service started: {}", service_name);
                self.log
                    .log(
                        LogLevel::Info,
                        "mcp_orchestrator",
                        &format!("MCP service started: {service_name}"),
                        Some(serde_json::json!({
                            "service_name": service_name,
                            "pid": process_id,
                            "port": port,
                        })),
                    )
                    .await
                    .ok();
            },
            McpEvent::ServiceFailed {
                service_name,
                error,
            } => {
                tracing::error!("Service failed: {} - {}", service_name, error);
                self.log
                    .log(
                        LogLevel::Error,
                        "mcp_orchestrator",
                        &format!("MCP service failed: {service_name}"),
                        Some(serde_json::json!({
                            "service_name": service_name,
                            "error": error,
                        })),
                    )
                    .await
                    .ok();
            },
            McpEvent::ServiceStopped {
                service_name,
                exit_code,
            } => {
                tracing::info!("Service stopped: {}", service_name);
                self.log
                    .log(
                        LogLevel::Info,
                        "mcp_orchestrator",
                        &format!("MCP service stopped: {service_name}"),
                        Some(serde_json::json!({
                            "service_name": service_name,
                            "exit_code": exit_code,
                        })),
                    )
                    .await
                    .ok();
            },
            McpEvent::HealthCheckFailed {
                service_name,
                reason,
            } => {
                tracing::warn!("Health check failed for {}: {}", service_name, reason);
                self.log
                    .log(
                        LogLevel::Warn,
                        "mcp_orchestrator",
                        &format!("Health check failed: {service_name}"),
                        Some(serde_json::json!({
                            "service_name": service_name,
                            "reason": reason,
                        })),
                    )
                    .await
                    .ok();
            },
            _ => {},
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "monitoring"
    }
}
