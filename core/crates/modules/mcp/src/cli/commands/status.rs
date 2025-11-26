use crate::services::McpOrchestrator;
use anyhow::Result;

pub async fn show_status(manager: &McpOrchestrator) -> Result<()> {
    manager.show_status().await
}
