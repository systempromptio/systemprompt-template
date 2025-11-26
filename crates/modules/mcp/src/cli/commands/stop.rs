use crate::services::McpOrchestrator;
use anyhow::Result;

pub async fn stop_services(manager: &McpOrchestrator, service_name: Option<String>) -> Result<()> {
    manager.stop_services(service_name).await
}
