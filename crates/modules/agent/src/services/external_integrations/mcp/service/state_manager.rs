use anyhow::Result;
use systemprompt_core_database::DbPool;

use systemprompt_models::repository::ServiceRepository;

#[derive(Debug, Clone)]
pub struct McpServiceState {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct ServiceStateManager {
    service_repo: ServiceRepository,
}

impl ServiceStateManager {
    pub const fn new(db_pool: DbPool) -> Self {
        Self {
            service_repo: ServiceRepository::new(db_pool),
        }
    }

    pub async fn get_mcp_service(&self, name: &str) -> Result<Option<McpServiceState>> {
        let service = self.service_repo.get_service_by_name(name).await?;
        Ok(service.map(|s| McpServiceState {
            name: s.name,
            host: "127.0.0.1".to_string(),
            port: s.port as u16,
            status: s.status,
        }))
    }

    pub async fn list_mcp_services(&self) -> Result<Vec<McpServiceState>> {
        let services = self.service_repo.get_mcp_services().await?;
        Ok(services
            .into_iter()
            .map(|s| McpServiceState {
                name: s.name,
                host: "127.0.0.1".to_string(),
                port: s.port as u16,
                status: s.status,
            })
            .collect())
    }

    pub async fn list_running_mcp_services(&self) -> Result<Vec<McpServiceState>> {
        let services = self.service_repo.get_mcp_services().await?;
        Ok(services
            .into_iter()
            .filter(|s| s.status == "running")
            .map(|s| McpServiceState {
                name: s.name,
                host: "127.0.0.1".to_string(),
                port: s.port as u16,
                status: s.status,
            })
            .collect())
    }
}
