use crate::services::McpOrchestrator;
use anyhow::Result;

pub async fn start_services(manager: &McpOrchestrator, service_name: Option<String>) -> Result<()> {
    manager.start_services(service_name).await
}
