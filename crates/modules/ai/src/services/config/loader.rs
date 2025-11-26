use anyhow::Result;
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::services::providers::provider_factory::AiProviderConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub default_provider: String,
    pub sampling: SamplingConfig,
    pub providers: HashMap<String, AiProviderConfig>,
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

        // Try to deserialize from the "ai" key first, then fallback to root
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
                "AI config file not found at: {}\nEnsure AI_CONFIG_PATH is set correctly and the file exists.",
                path.display()
            ));
        }

        Self::load_from_file(path)
    }

    pub fn default_config() -> AiConfig {
        let mut providers = HashMap::new();

        providers.insert(
            "openai".to_string(),
            AiProviderConfig {
                enabled: false,
                api_key: String::new(),
                endpoint: Some("https://api.openai.com/v1".to_string()),
                default_model: "gpt-4-turbo".to_string(),
                google_search_enabled: false,
            },
        );

        providers.insert(
            "anthropic".to_string(),
            AiProviderConfig {
                enabled: false,
                api_key: String::new(),
                endpoint: Some("https://api.anthropic.com/v1".to_string()),
                default_model: "claude-3-sonnet-20240229".to_string(),
                google_search_enabled: false,
            },
        );

        providers.insert(
            "gemini".to_string(),
            AiProviderConfig {
                enabled: false,
                api_key: String::new(),
                endpoint: Some("https://generativelanguage.googleapis.com/v1beta".to_string()),
                default_model: "gemini-2.5-flash-lite".to_string(),
                google_search_enabled: false,
            },
        );

        AiConfig {
            default_provider: "anthropic".to_string(),
            sampling: SamplingConfig {
                enable_smart_routing: true,
                fallback_enabled: true,
            },
            providers,
            mcp: McpConfig {
                auto_discover: false,
                connect_timeout_ms: 5000,
                execution_timeout_ms: 30000,
                retry_attempts: 3,
            },
            history: HistoryConfig {
                retention_days: 30,
                log_tool_executions: true,
            },
        }
    }

    pub fn expand_env_vars(config: &mut AiConfig) -> Result<()> {
        for provider_config in config.providers.values_mut() {
            if provider_config.api_key.starts_with("${") && provider_config.api_key.ends_with('}') {
                let var_name = &provider_config.api_key[2..provider_config.api_key.len() - 1];
                if let Ok(value) = std::env::var(var_name) {
                    provider_config.api_key = value;
                } else {
                    // If API key not found, disable the provider instead of failing
                    provider_config.enabled = false;
                    provider_config.api_key = String::new();
                }
            }
        }
        Ok(())
    }
}
