use anyhow::Result;
use async_trait::async_trait;

use crate::services::lifecycle::LifecycleManager;
use crate::services::registry::RegistryManager;

use super::{EventHandler, McpEvent};

#[derive(Debug)]
pub struct LifecycleHandler {
    lifecycle: LifecycleManager,
    registry: RegistryManager,
}

impl LifecycleHandler {
    pub const fn new(lifecycle: LifecycleManager, registry: RegistryManager) -> Self {
        Self {
            lifecycle,
            registry,
        }
    }
}

#[async_trait]
impl EventHandler for LifecycleHandler {
    async fn handle(&self, event: &McpEvent) -> Result<()> {
        match event {
            McpEvent::ServiceStartRequested { service_name } => {
                let config = self.registry.get_server(service_name).await?;

                tracing::info!("Starting MCP service: {}", service_name);

                self.lifecycle.start_server(&config).await?;
            },
            McpEvent::ServiceStopped {
                service_name,
                exit_code,
            } => {
                tracing::info!(
                    "Service {} stopped with exit code {:?}",
                    service_name,
                    exit_code
                );
            },
            McpEvent::ServiceRestartRequested {
                service_name,
                reason,
            } => {
                tracing::info!("Restarting service {} due to: {}", service_name, reason);

                let config = self.registry.get_server(service_name).await?;

                self.lifecycle.restart_server(&config).await?;
            },
            _ => {},
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "lifecycle"
    }

    fn handles(&self, event: &McpEvent) -> bool {
        matches!(
            event,
            McpEvent::ServiceStartRequested { .. }
                | McpEvent::ServiceStopped { .. }
                | McpEvent::ServiceRestartRequested { .. }
        )
    }
}
