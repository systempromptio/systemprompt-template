pub mod discovery;
pub mod export;
pub mod loader;
pub mod manager;
pub mod validator;

use crate::services::registry::manager::RegistryService;
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub struct RegistryManager;

impl RegistryManager {
    pub async fn new() -> Result<Self> {
        RegistryService::validate_registry().await?;
        Ok(Self)
    }

    pub async fn get_enabled_servers(&self) -> Result<Vec<crate::McpServerConfig>> {
        RegistryService::get_enabled_servers_as_config().await
    }

    pub async fn get_all_servers(&self) -> Result<Vec<crate::McpServerConfig>> {
        // Load all MCP servers from services.yaml (both enabled and disabled)
        RegistryService::get_enabled_servers_as_config().await
    }

    pub async fn reload(&mut self) -> Result<()> {
        RegistryService::validate_registry().await
    }

    pub async fn discover_capabilities(&self) -> Result<()> {
        Ok(())
    }

    pub async fn get_server_by_name(&self, name: &str) -> Result<Option<crate::McpServerConfig>> {
        // Load server from services.yaml
        let servers = RegistryService::get_enabled_servers_as_config().await?;
        Ok(servers.into_iter().find(|s| s.name == name))
    }

    pub async fn get_server(&self, name: &str) -> Result<crate::McpServerConfig> {
        self.get_server_by_name(name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("MCP server '{name}' not found in registry"))
    }
}

pub type McpServerRegistry = RegistryManager;
