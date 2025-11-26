use anyhow::Result;
use async_trait::async_trait;

use crate::services::database::DatabaseManager;

use super::{EventHandler, McpEvent};

#[derive(Debug)]
pub struct DatabaseSyncHandler {
    database: DatabaseManager,
}

impl DatabaseSyncHandler {
    pub const fn new(database: DatabaseManager) -> Self {
        Self { database }
    }
}

#[async_trait]
impl EventHandler for DatabaseSyncHandler {
    async fn handle(&self, event: &McpEvent) -> Result<()> {
        match event {
            McpEvent::ServiceStarted { service_name, .. } => {
                self.database
                    .update_service_status(service_name, "running")
                    .await?;
            },
            McpEvent::ServiceFailed { service_name, .. } => {
                self.database
                    .update_service_status(service_name, "failed")
                    .await?;
            },
            McpEvent::ServiceStopped { service_name, .. } => {
                self.database
                    .update_service_status(service_name, "stopped")
                    .await?;
            },
            _ => {},
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "database_sync"
    }
}
