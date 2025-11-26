use anyhow::{anyhow, Result};
use systemprompt_core_config::ConfigLoader;
use systemprompt_models::ServicesConfig;

use crate::Deployment;

#[derive(Debug, Clone, Copy)]
pub struct DeploymentService;

impl DeploymentService {
    pub async fn load_config() -> Result<ServicesConfig> {
        ConfigLoader::load().await
    }

    pub async fn get_deployment(&self, name: &str) -> Result<Deployment> {
        let config = Self::load_config().await?;
        config
            .mcp_servers
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow!("No deployment configuration found for server: {name}"))
    }

    pub async fn list_enabled_servers() -> Result<Vec<String>> {
        let config = Self::load_config().await?;
        let enabled: Vec<String> = config
            .mcp_servers
            .iter()
            .filter_map(|(name, deployment)| {
                if deployment.enabled {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        Ok(enabled)
    }

    pub async fn get_server_port(name: &str) -> Result<u16> {
        let config = Self::load_config().await?;
        config
            .mcp_servers
            .get(name)
            .map(|d| d.port)
            .ok_or_else(|| anyhow!("No deployment configuration found for server: {name}"))
    }

    pub async fn is_server_enabled(name: &str) -> Result<bool> {
        let config = Self::load_config().await?;
        config
            .mcp_servers
            .get(name)
            .map(|d| d.enabled)
            .ok_or_else(|| anyhow!("No deployment configuration found for server: {name}"))
    }

    pub async fn validate_config() -> Result<()> {
        let config = ConfigLoader::load().await?;
        config.validate()?;
        Ok(())
    }

    pub async fn get_server_binary(name: &str) -> Result<String> {
        let config = Self::load_config().await?;
        config
            .mcp_servers
            .get(name)
            .and_then(|d| d.binary.clone())
            .or_else(|| Some(name.to_string()))
            .ok_or_else(|| anyhow!("No deployment configuration found for server: {name}"))
    }

    pub async fn get_server_package(name: &str) -> Result<String> {
        let config = Self::load_config().await?;
        config
            .mcp_servers
            .get(name)
            .and_then(|d| d.package.clone())
            .or_else(|| Some(name.to_string()))
            .ok_or_else(|| anyhow!("No deployment configuration found for server: {name}"))
    }
}
