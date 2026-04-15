use serde::Deserialize;
use systemprompt::identifiers::{AgentId, McpServerId, SkillId};

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub id: AgentId,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default = "super::plugins::default_true")]
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub system_prompt: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMcpRequest {
    pub id: McpServerId,
    #[serde(default = "super::plugins::default_external")]
    pub server_type: String,
    #[serde(default)]
    pub binary: String,
    #[serde(default)]
    pub package_name: String,
    #[serde(default = "super::plugins::default_port")]
    pub port: u16,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "super::plugins::default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub oauth_required: bool,
    #[serde(default)]
    pub oauth_scopes: Vec<String>,
    #[serde(default)]
    pub oauth_audience: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMcpRequest {
    pub server_type: Option<String>,
    pub binary: Option<String>,
    pub package_name: Option<String>,
    pub port: Option<u16>,
    pub endpoint: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub oauth_required: Option<bool>,
    pub oauth_scopes: Option<Vec<String>>,
    pub oauth_audience: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMcpRawYamlRequest {
    pub yaml_content: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateHookRequest {
    pub plugin_id: String,
    pub event: String,
    pub matcher: String,
    pub command: String,
    #[serde(default)]
    pub is_async: bool,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateHookRequest {
    pub plugin_id: Option<String>,
    pub event: Option<String>,
    pub matcher: Option<String>,
    pub command: Option<String>,
    pub is_async: Option<bool>,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePluginRequest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "super::plugins::default_version")]
    pub version: String,
    #[serde(default = "super::plugins::default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub author_name: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub skills: Vec<SkillId>,
    #[serde(default)]
    pub agents: Vec<AgentId>,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerId>,
    #[serde(default)]
    pub hooks: Vec<CreateHookRequest>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePluginRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub enabled: Option<bool>,
    pub category: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub author_name: Option<String>,
    pub roles: Option<Vec<String>>,
    pub skills: Option<Vec<SkillId>>,
    pub agents: Option<Vec<AgentId>>,
    pub mcp_servers: Option<Vec<McpServerId>>,
    pub hooks: Option<Vec<CreateHookRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct UserQuery {
    pub user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePluginSkillsRequest {
    pub skills: Vec<SkillId>,
}

#[derive(Debug, Deserialize)]
pub struct EnvVarEntry {
    pub var_name: String,
    pub var_value: String,
    #[serde(default)]
    pub is_secret: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePluginEnvRequest {
    pub variables: Vec<EnvVarEntry>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSkillFileRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ImportPluginRequest {
    pub url: String,
    #[serde(default)]
    pub import_target: Option<String>,
}
