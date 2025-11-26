use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;

use crate::models::ai::{ModelHint, ModelPreferences, SamplingMetadata};
use crate::services::providers::AiProvider;

pub struct SamplingRouter {
    providers: HashMap<String, Arc<dyn AiProvider>>,
    default_provider: String,
}

impl std::fmt::Debug for SamplingRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SamplingRouter")
            .field("default_provider", &self.default_provider)
            .finish_non_exhaustive()
    }
}

impl SamplingRouter {
    pub fn new(providers: HashMap<String, Arc<dyn AiProvider>>, default_provider: String) -> Self {
        Self {
            providers,
            default_provider,
        }
    }

    pub fn select_provider(
        &self,
        preferences: &ModelPreferences,
        metadata: &SamplingMetadata,
    ) -> Result<(String, Arc<dyn AiProvider>)> {
        let provider_scores = self.calculate_provider_scores(preferences, metadata)?;

        let (best_provider, _) = provider_scores
            .iter()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .ok_or_else(|| anyhow!("No suitable provider found"))?;

        let provider = self
            .providers
            .get(best_provider.as_str())
            .ok_or_else(|| anyhow!("Provider {best_provider} not found"))?;

        Ok((best_provider.clone(), provider.clone()))
    }

    pub fn select_model(
        &self,
        provider_name: &str,
        preferences: &ModelPreferences,
    ) -> Result<String> {
        let provider = self
            .providers
            .get(provider_name)
            .ok_or_else(|| anyhow!("Provider {provider_name} not found"))?;

        for hint in &preferences.hints {
            match hint {
                ModelHint::ModelId(model_id) => {
                    if provider.supports_model(model_id) {
                        return Ok(model_id.clone());
                    }
                },
                ModelHint::Category(category) => {
                    let model = self.select_model_by_category(
                        provider_name,
                        category,
                        preferences.cost_priority,
                    )?;
                    if provider.supports_model(&model) {
                        return Ok(model);
                    }
                },
                ModelHint::Provider(hint_provider) => {
                    if hint_provider == provider_name {
                        return Ok(provider.default_model().to_string());
                    }
                },
            }
        }

        Ok(provider.default_model().to_string())
    }

    fn calculate_provider_scores(
        &self,
        preferences: &ModelPreferences,
        metadata: &SamplingMetadata,
    ) -> Result<HashMap<String, f32>> {
        let mut scores = HashMap::new();

        for (name, provider) in &self.providers {
            let mut score = 0.0;

            if provider.supports_metadata(metadata) {
                score += 1.0;
            }

            for hint in &preferences.hints {
                match hint {
                    ModelHint::Provider(hint_provider) => {
                        if hint_provider == name {
                            score += 3.0;
                        }
                    },
                    ModelHint::ModelId(model_id) => {
                        if provider.supports_model(model_id) {
                            score += 2.0;
                        }
                    },
                    ModelHint::Category(category) => {
                        if Self::provider_supports_category(name, category) {
                            score += 1.5;
                        }
                    },
                }
            }

            if let Some(cost_priority) = preferences.cost_priority {
                score += Self::calculate_cost_score(name, cost_priority);
            }

            scores.insert(name.clone(), score);
        }

        if scores.is_empty() {
            scores.insert(self.default_provider.clone(), 1.0);
        }

        Ok(scores)
    }

    fn select_model_by_category(
        &self,
        provider: &str,
        category: &str,
        cost_priority: Option<f32>,
    ) -> Result<String> {
        let cost_priority = cost_priority.unwrap_or(0.5);

        match (provider, category) {
            ("openai", "fast") => Ok("gpt-3.5-turbo".to_string()),
            ("openai", "quality") => Ok("gpt-4-turbo".to_string()),
            ("anthropic", "fast") => Ok("claude-3-haiku-20240307".to_string()),
            ("anthropic", "quality") => {
                if cost_priority < 0.3 {
                    Ok("claude-3-opus-20240229".to_string())
                } else {
                    Ok("claude-3-sonnet-20240229".to_string())
                }
            },
            ("gemini", "fast") => Ok("gemini-2.5-flash-lite".to_string()),
            ("gemini", "quality" | "vision") => Ok("gemini-2.5-flash".to_string()),
            _ => Ok(self
                .providers
                .get(provider).map_or_else(|| "gpt-4-turbo".to_string(), |p| p.default_model().to_string())),
        }
    }

    fn provider_supports_category(provider: &str, category: &str) -> bool {
        matches!(
            (provider, category),
            ("openai" | "anthropic", "fast" | "quality")
                | ("gemini", "fast" | "quality" | "vision")
        )
    }

    fn calculate_cost_score(provider: &str, cost_priority: f32) -> f32 {
        let provider_costs = match provider {
            "openai" => 1.0,
            "anthropic" => 0.8,
            "gemini" => 0.3,
            _ => 0.5,
        };

        (1.0 - cost_priority) * (1.0 - provider_costs)
    }
}
