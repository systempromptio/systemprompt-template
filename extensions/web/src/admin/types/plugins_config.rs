use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt::models::{PluginConfig, PluginVariableDef};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginOnboardingQuestion {
    pub question: String,
    pub listen_for: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginOnboardingDataSource {
    pub mcp_server: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub connection_question: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginOnboardingConfig {
    #[serde(default)]
    pub interview_questions: Vec<PluginOnboardingQuestion>,
    #[serde(default)]
    pub data_sources: Vec<PluginOnboardingDataSource>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlatformPluginConfig {
    #[serde(flatten)]
    pub base: PluginConfig,

    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub depends: Vec<String>,
    #[serde(default)]
    pub variables: Vec<PluginVariableDef>,
    #[serde(default)]
    pub onboarding: Option<PluginOnboardingConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlatformPluginConfigFile {
    pub plugin: PlatformPluginConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginOverview {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub skills: Vec<SkillInfo>,
    pub agents: Vec<AgentInfo>,
    pub mcp_servers: Vec<String>,
    pub hooks: Vec<HookOverview>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HookOverview {
    pub event: String,
    pub matcher: String,
    pub command: String,
    pub is_async: bool,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "super::plugins::default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredSecret {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub command: String,
    pub source: String,
    #[serde(default = "super::plugins::default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub required_secrets: Vec<RequiredSecret>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default = "super::plugins::default_true")]
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentSkillInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentDetail {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub is_primary: bool,
    #[serde(default)]
    pub show_in_ui: bool,
    pub system_prompt: String,
    pub port: Option<u16>,
    pub endpoint: Option<String>,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    #[serde(default)]
    pub skills: Vec<AgentSkillInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpServerDetail {
    pub id: String,
    #[serde(default = "super::plugins::default_internal")]
    pub server_type: String,
    pub binary: String,
    pub package_name: String,
    pub port: u16,
    pub endpoint: String,
    pub description: String,
    pub enabled: bool,
    pub oauth_required: bool,
    pub oauth_scopes: Vec<String>,
    pub oauth_audience: String,
    #[serde(default = "super::plugins::default_true")]
    pub removable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HookCatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub event: String,
    pub matcher: String,
    pub command: String,
    pub is_async: bool,
    pub category: String,
    pub enabled: bool,
    pub tags: Vec<String>,
    pub visible_to: Vec<String>,
    pub checksum: String,
    #[serde(default)]
    #[sqlx(skip)]
    pub plugins: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HookDetail {
    pub id: String,
    pub plugin_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub event: String,
    pub matcher: String,
    pub command: String,
    pub is_async: bool,
    #[serde(default)]
    pub system: bool,
    #[serde(default)]
    pub visible_to: Vec<String>,
    #[serde(default = "super::plugins::default_true")]
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginDetail {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub enabled: bool,
    pub category: String,
    pub keywords: Vec<String>,
    pub author_name: String,
    pub roles: Vec<String>,
    pub skills: Vec<String>,
    pub agents: Vec<String>,
    pub mcp_servers: Vec<String>,
}
