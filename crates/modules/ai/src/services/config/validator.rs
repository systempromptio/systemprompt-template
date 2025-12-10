use super::loader::AiConfig;
use anyhow::{anyhow, Result};
use systemprompt_core_logging::LogService;

#[derive(Debug, Copy, Clone)]
pub struct ConfigValidator;

impl ConfigValidator {
    pub async fn validate(config: &AiConfig, log: &LogService) -> Result<()> {
        Self::log_config_source(log).await;
        Self::validate_providers(config)?;
        Self::validate_sampling(config, log).await?;
        Self::validate_mcp(config, log).await?;
        Self::validate_history(config, log).await?;
        Ok(())
    }

    async fn log_config_source(log: &LogService) {
        if let Ok(path) = std::env::var("AI_CONFIG_PATH") {
            log.info("ai_config", &format!("Config loaded from: {path}"))
                .await
                .ok();
        } else {
            log.warn("ai_config", "AI_CONFIG_PATH not set, using search fallback")
                .await
                .ok();
        }
    }

    fn validate_providers(config: &AiConfig) -> Result<()> {
        let enabled_providers: Vec<_> =
            config.providers.iter().filter(|(_, c)| c.enabled).collect();

        if enabled_providers.is_empty() {
            return Err(anyhow!(
                "No AI providers are enabled. Check your config file:\n- Ensure at least one \
                 provider has 'enabled: true'\n- Verify API keys are set (GEMINI_API_KEY, \
                 ANTHROPIC_API_KEY, or OPENAI_API_KEY in .env)\n- Current providers defined: {:?}",
                config.providers.keys().collect::<Vec<_>>()
            ));
        }

        for (name, provider_config) in &enabled_providers {
            if provider_config.api_key.is_empty() {
                return Err(anyhow!(
                    "Provider '{}' is enabled but has no API key.\nFix: Set {}_API_KEY in your \
                     .env file",
                    name,
                    name.to_uppercase()
                ));
            }

            if provider_config.default_model.is_empty() {
                return Err(anyhow!("Provider {name} has no default model specified"));
            }
        }

        if !config.providers.contains_key(&config.default_provider) {
            return Err(anyhow!(
                "Default provider '{}' not found in providers.\nAvailable providers: {:?}\nFix: \
                 Update 'default_provider' in your config file",
                config.default_provider,
                config.providers.keys().collect::<Vec<_>>()
            ));
        }

        if !config.providers[&config.default_provider].enabled {
            let available: Vec<&str> = config
                .providers
                .iter()
                .filter(|(_, c)| c.enabled)
                .map(|(n, _)| n.as_str())
                .collect();

            return Err(anyhow!(
                "Default provider '{}' is not enabled.\nEnabled providers: {:?}\nFix: Either \
                 enable '{}' in your config OR change 'default_provider' to one of the enabled \
                 providers",
                config.default_provider,
                available,
                config.default_provider
            ));
        }

        Ok(())
    }

    async fn validate_sampling(config: &AiConfig, log: &LogService) -> Result<()> {
        if !config.sampling.enable_smart_routing && !config.sampling.fallback_enabled {
            let _ = log
                .warn("ai_config", "Both smart routing and fallback are disabled")
                .await;
        }
        Ok(())
    }

    async fn validate_mcp(config: &AiConfig, log: &LogService) -> Result<()> {
        if config.mcp.connect_timeout_ms == 0 {
            return Err(anyhow!("MCP connect timeout must be greater than 0"));
        }

        if config.mcp.execution_timeout_ms == 0 {
            return Err(anyhow!("MCP execution timeout must be greater than 0"));
        }

        if config.mcp.retry_attempts == 0 {
            let _ = log
                .warn(
                    "ai_config",
                    "MCP retry attempts set to 0, failures will not be retried",
                )
                .await;
        }

        Ok(())
    }

    async fn validate_history(config: &AiConfig, log: &LogService) -> Result<()> {
        if config.history.retention_days == 0 {
            let _ = log
                .warn(
                    "ai_config",
                    "History retention set to 0 days, history will not be retained",
                )
                .await;
        }

        if config.history.retention_days > 365 {
            let _ = log
                .warn(
                    "ai_config",
                    &format!(
                        "History retention set to {} days, consider reducing for storage \
                         efficiency",
                        config.history.retention_days
                    ),
                )
                .await;
        }

        Ok(())
    }
}
