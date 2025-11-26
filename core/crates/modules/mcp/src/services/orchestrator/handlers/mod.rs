use anyhow::Result;
use async_trait::async_trait;

use super::events::McpEvent;

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &McpEvent) -> Result<()>;

    fn name(&self) -> &'static str;

    fn handles(&self, event: &McpEvent) -> bool {
        let _ = event;
        true
    }
}

pub mod database_sync;
pub mod health_check;
pub mod lifecycle;
pub mod monitoring;

pub use database_sync::DatabaseSyncHandler;
pub use health_check::HealthCheckHandler;
pub use lifecycle::LifecycleHandler;
pub use monitoring::MonitoringHandler;
