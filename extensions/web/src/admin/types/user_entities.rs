use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt::identifiers::{AgentId, McpServerId, SkillId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPlugin {
    pub id: String,
    pub user_id: UserId,
    pub plugin_id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub enabled: bool,
    pub category: String,
    pub keywords: Vec<String>,
    pub author_name: String,
    pub base_plugin_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserMcpServer {
    pub id: String,
    pub user_id: UserId,
    pub mcp_server_id: McpServerId,
    pub name: String,
    pub description: String,
    pub binary: String,
    pub package_name: String,
    pub port: i32,
    pub endpoint: String,
    pub enabled: bool,
    pub oauth_required: bool,
    pub oauth_scopes: Vec<String>,
    pub oauth_audience: String,
    pub base_mcp_server_id: Option<McpServerId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserPluginRequest {
    pub plugin_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub author_name: String,
    #[serde(default)]
    pub base_plugin_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserPluginRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub enabled: Option<bool>,
    pub category: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub author_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserMcpServerRequest {
    pub mcp_server_id: McpServerId,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub binary: String,
    #[serde(default)]
    pub package_name: String,
    #[serde(default)]
    pub port: i32,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub oauth_required: bool,
    #[serde(default)]
    pub oauth_scopes: Vec<String>,
    #[serde(default)]
    pub oauth_audience: String,
    #[serde(default)]
    pub base_mcp_server_id: Option<McpServerId>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserMcpServerRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub binary: Option<String>,
    pub package_name: Option<String>,
    pub port: Option<i32>,
    pub endpoint: Option<String>,
    pub enabled: Option<bool>,
    pub oauth_required: Option<bool>,
    pub oauth_scopes: Option<Vec<String>>,
    pub oauth_audience: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPluginWithAssociations {
    pub plugin: UserPlugin,
    pub skill_ids: Vec<SkillId>,
    pub agent_ids: Vec<AgentId>,
    pub mcp_server_ids: Vec<McpServerId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MarketplaceSyncStatus {
    pub user_id: UserId,
    pub dirty: bool,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_changed_at: DateTime<Utc>,
    pub sync_error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ForkPluginRequest {
    pub org_plugin_id: String,
    #[serde(default)]
    pub plugin_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForkSkillRequest {
    pub org_skill_id: SkillId,
    #[serde(default)]
    pub skill_id: Option<SkillId>,
}

#[derive(Debug, Deserialize)]
pub struct ForkAgentRequest {
    pub org_agent_id: AgentId,
    #[serde(default)]
    pub agent_id: Option<AgentId>,
}

#[derive(Debug, Deserialize)]
pub struct ForkMcpServerRequest {
    pub org_mcp_server_id: McpServerId,
    #[serde(default)]
    pub mcp_server_id: Option<McpServerId>,
}

#[derive(Debug, Deserialize)]
pub struct ForkHookRequest {
    pub org_hook_id: String,
    #[serde(default)]
    pub hook_id: Option<String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}
