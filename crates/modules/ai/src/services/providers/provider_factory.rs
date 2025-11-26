use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use systemprompt_core_database::DbPool;

use super::{AiProvider, AnthropicProvider, GeminiProvider, OpenAiProvider};

/// Configuration for AI provider initialization (`OpenAI`, Anthropic, Gemini).
/// Contains API credentials and endpoint settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    pub enabled: bool,
    pub api_key: String,
    pub endpoint: Option<String>,
    pub default_model: String,
    #[serde(default)]
    pub google_search_enabled: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(
        name: &str,
        config: &AiProviderConfig,
        db_pool: Option<DbPool>,
    ) -> Result<Arc<dyn AiProvider>> {
        if !config.enabled {
            return Err(anyhow!("Provider {name} is disabled"));
        }

        let provider: Arc<dyn AiProvider> = match name {
            "openai" => {
                if let Some(endpoint) = &config.endpoint {
                    Arc::new(OpenAiProvider::with_endpoint(
                        config.api_key.clone(),
                        endpoint.clone(),
                    ))
                } else {
                    Arc::new(OpenAiProvider::new(config.api_key.clone()))
                }
            },
            "anthropic" => {
                if let Some(endpoint) = &config.endpoint {
                    Arc::new(AnthropicProvider::with_endpoint(
                        config.api_key.clone(),
                        endpoint.clone(),
                    ))
                } else {
                    Arc::new(AnthropicProvider::new(config.api_key.clone()))
                }
            },
            "gemini" => {
                let mut provider = if let Some(endpoint) = &config.endpoint {
                    GeminiProvider::with_endpoint(config.api_key.clone(), endpoint.clone())
                } else {
                    GeminiProvider::new(config.api_key.clone())
                };

                if config.google_search_enabled {
                    provider = provider.with_google_search();
                }

                if let Some(pool) = db_pool {
                    provider = provider.with_db_pool(pool);
                }

                Arc::new(provider)
            },
            _ => return Err(anyhow!("Unknown provider: {name}")),
        };

        Ok(provider)
    }

    pub fn create_all(
        configs: HashMap<String, AiProviderConfig>,
        db_pool: Option<&DbPool>,
    ) -> Result<HashMap<String, Arc<dyn AiProvider>>> {
        let mut providers = HashMap::new();

        for (name, config) in configs {
            if config.enabled {
                match Self::create(&name, &config, db_pool.cloned()) {
                    Ok(provider) => {
                        providers.insert(name, provider);
                    },
                    Err(_e) => {
                        // Provider creation failed, skip this provider
                    },
                }
            }
        }

        if providers.is_empty() {
            return Err(anyhow!("No providers could be initialized"));
        }

        Ok(providers)
    }
}
