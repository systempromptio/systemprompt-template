use crate::services::McpOrchestrator;
use anyhow::Result;

pub async fn list_services(manager: &McpOrchestrator) -> Result<()> {
    manager.list_services().await
}
