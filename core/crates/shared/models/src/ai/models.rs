use serde::{Deserialize, Serialize};

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
