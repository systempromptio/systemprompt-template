use anyhow::Result;
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::services::providers::provider_factory::AiProviderConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolModelSettings {
    pub model: String,
    #[serde(default)]
    pub max_output_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub default_provider: String,
    #[serde(default)]
    pub default_max_output_tokens: Option<u32>,
    pub sampling: SamplingConfig,
    pub providers: HashMap<String, AiProviderConfig>,
    #[serde(default)]
    pub tool_models: HashMap<String, ToolModelSettings>,
    pub mcp: McpConfig,
    pub history: HistoryConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SamplingConfig {
    pub enable_smart_routing: bool,
    pub fallback_enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct McpConfig {
    pub auto_discover: bool,
    pub connect_timeout_ms: u64,
    pub execution_timeout_ms: u64,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HistoryConfig {
    pub retention_days: u32,
    pub log_tool_executions: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load_from_file(path: &Path) -> Result<AiConfig> {
        let settings = Config::builder()
            .add_source(File::from(path))
            .add_source(Environment::with_prefix("AI").separator("__"))
            .build()?;

        let config: AiConfig = if let Ok(ai_section) = settings.get::<AiConfig>("ai") {
            ai_section
        } else {
            settings.try_deserialize()?
        };

        Ok(config)
    }

    pub fn load_default() -> Result<AiConfig> {
        let config_path = std::env::var("AI_CONFIG_PATH")
            .map_err(|_| anyhow::anyhow!("AI_CONFIG_PATH environment variable is not set"))?;

        let path = Path::new(&config_path);
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "AI config file not found at: {}\nEnsure AI_CONFIG_PATH is set correctly and the \
                 file exists.",
                path.display()
            ));
        }

        Self::load_from_file(path)
    }

    pub fn expand_env_vars(config: &mut AiConfig) -> Result<()> {
        for provider_config in config.providers.values_mut() {
            if provider_config.api_key.starts_with("${") && provider_config.api_key.ends_with('}') {
                let var_name = &provider_config.api_key[2..provider_config.api_key.len() - 1];
                if let Ok(value) = std::env::var(var_name) {
                    provider_config.api_key = value;
                } else {
                    provider_config.enabled = false;
                    provider_config.api_key = String::new();
                }
            }
        }
        Ok(())
    }
}
