use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use systemprompt_models::{Config, PartialServicesConfig, ServicesConfig, SystemPaths};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigWithIncludes {
    #[serde(default)]
    includes: Vec<String>,
    #[serde(flatten)]
    config: ServicesConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigLoader;

impl ConfigLoader {
    fn default_config_path() -> String {
        let config = Config::global();
        SystemPaths::services_config(config)
            .to_string_lossy()
            .into_owned()
    }

    pub async fn load() -> Result<ServicesConfig> {
        Self::load_from_path(&Self::default_config_path()).await
    }

    async fn load_from_path(config_path: &str) -> Result<ServicesConfig> {
        let content = fs::read_to_string(config_path).with_context(|| {
            format!(
                "Failed to read services config file: {config_path}\nEnsure SYSTEM_PATH \
                 environment variable is correctly set to the core directory.\nServices config \
                 should be at: $SYSTEM_PATH/crates/services/config/config.yml"
            )
        })?;

        let root_config: ConfigWithIncludes = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse services config YAML: {config_path}"))?;

        let config_dir = Path::new(config_path)
            .parent()
            .unwrap_or_else(|| Path::new("."));

        let mut merged_config = root_config.config;

        for include_path in &root_config.includes {
            let full_path = config_dir.join(include_path);
            let include_config = Self::load_include_file(&full_path).await?;

            for (name, agent) in include_config.agents {
                merged_config.agents.insert(name, agent);
            }

            for (name, mcp_server) in include_config.mcp_servers {
                merged_config.mcp_servers.insert(name, mcp_server);
            }

            if include_config.scheduler.is_some() {
                merged_config.scheduler = include_config.scheduler;
            }
        }

        merged_config
            .validate()
            .with_context(|| "Services config validation failed")?;

        Ok(merged_config)
    }

    async fn load_include_file(path: &PathBuf) -> Result<PartialServicesConfig> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read included config file: {}", path.display()))?;

        let config: PartialServicesConfig = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse included config YAML: {}", path.display()))?;

        Ok(config)
    }

    pub async fn validate_file(path: &str) -> Result<()> {
        let config = Self::load_from_path(path).await?;
        config
            .validate()
            .with_context(|| "Config validation failed")?;
        Ok(())
    }
}
