use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ToolModelConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
}

impl ToolModelConfig {
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: Some(provider.into()),
            model: Some(model.into()),
            max_output_tokens: None,
        }
    }

    pub const fn with_max_output_tokens(mut self, tokens: u32) -> Self {
        self.max_output_tokens = Some(tokens);
        self
    }

    pub const fn is_empty(&self) -> bool {
        self.provider.is_none() && self.model.is_none() && self.max_output_tokens.is_none()
    }

    pub fn merge_with(&self, other: &Self) -> Self {
        Self {
            provider: other.provider.clone().or_else(|| self.provider.clone()),
            model: other.model.clone().or_else(|| self.model.clone()),
            max_output_tokens: other.max_output_tokens.or(self.max_output_tokens),
        }
    }
}

pub type ToolModelOverrides = HashMap<String, HashMap<String, ToolModelConfig>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub max_tokens: u32,
    pub supports_tools: bool,
    #[serde(default)]
    pub cost_per_1k_tokens: f32,
}

impl ModelConfig {
    pub fn new(id: impl Into<String>, max_tokens: u32, supports_tools: bool) -> Self {
        Self {
            id: id.into(),
            max_tokens,
            supports_tools,
            cost_per_1k_tokens: 0.0,
        }
    }

    pub const fn with_cost(mut self, cost: f32) -> Self {
        self.cost_per_1k_tokens = cost;
        self
    }
}
