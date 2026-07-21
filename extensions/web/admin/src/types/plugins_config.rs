//! Deserialisation model for an installed plugin's configuration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::{AgentId, HookId, McpServerId, PluginId, SkillId};
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

impl PlatformPluginConfig {
    pub const fn from_base(base: PluginConfig) -> Self {
        Self {
            base,
            roles: Vec::new(),
            depends: Vec::new(),
            variables: Vec::new(),
            onboarding: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginOverview {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub skills: Vec<SkillInfo>,
    pub agents: Vec<AgentInfo>,
    pub mcp_servers: Vec<McpServerId>,
    pub hooks: Vec<HookOverview>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfiguredHook {
    pub id: String,
    pub plugin_id: PluginId,
    pub event: String,
    pub matcher: String,
    pub command: String,
    pub is_async: bool,
    pub timeout_ms: Option<u32>,
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
    #[serde(default = "default_hook_id")]
    pub id: HookId,
}

fn default_hook_id() -> HookId {
    HookId::new("")
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
    pub id: SkillId,
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
    pub id: AgentId,
    pub name: String,
    pub description: String,
    #[serde(default = "super::plugins::default_true")]
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentSkillInfo {
    pub id: SkillId,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentDetail {
    pub id: AgentId,
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
    pub mcp_servers: Vec<McpServerId>,
    #[serde(default)]
    pub skills: Vec<AgentSkillInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCatalogEntry {
    pub id: SkillId,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCatalogEntry {
    pub id: AgentId,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub source_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpServerDetail {
    pub id: McpServerId,
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
    /// YAML file this entry was loaded from, relative to the services directory
    /// (e.g. `services/mcp/openai.yaml`). Empty when the entry was just created
    /// via a mutation and not yet re-read from disk.
    #[serde(default)]
    pub source_path: String,
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
    pub id: HookId,
    pub plugin_id: PluginId,
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
    pub skills: Vec<SkillId>,
    pub agents: Vec<AgentId>,
    pub mcp_servers: Vec<McpServerId>,
    /// YAML file this entry was loaded from, relative to the services directory
    /// (e.g. `services/plugins/enterprise-demo/config.yaml`). Empty for
    /// in-memory entries.
    #[serde(default)]
    pub source_path: String,
}
