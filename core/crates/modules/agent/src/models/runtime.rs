use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    /// Skill IDs to load and inject into agent system prompt
    #[serde(default)]
    pub skills: Vec<String>,
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
        }
    }
}
