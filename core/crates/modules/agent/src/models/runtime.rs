use serde::{Deserialize, Serialize};
use systemprompt_models::ai::ToolModelOverrides;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentRuntimeInfo {
    pub name: String,
    pub port: u16,
    pub is_enabled: bool,
    pub is_primary: bool,
    pub system_prompt: Option<String>,
    pub mcp_servers: Vec<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub tool_model_overrides: ToolModelOverrides,
}

impl From<systemprompt_models::AgentConfig> for AgentRuntimeInfo {
    fn from(config: systemprompt_models::AgentConfig) -> Self {
        Self {
            name: config.name,
            port: config.port,
            is_enabled: config.enabled,
            is_primary: config.is_primary,
            system_prompt: config.metadata.system_prompt,
            mcp_servers: config.metadata.mcp_servers,
            provider: config.metadata.provider,
            model: config.metadata.model,
            skills: config.metadata.skills,
            tool_model_overrides: config.metadata.tool_model_overrides,
        }
    }
}
