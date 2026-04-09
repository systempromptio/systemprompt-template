use serde::Serialize;

use super::common::NamedEntity;

#[derive(Debug, Clone, Serialize)]
pub struct McpServerView {
    pub id: String,
    pub mcp_server_id: String,
    pub name: String,
    pub description: String,
    pub endpoint: String,
    pub enabled: bool,
    pub plugin_names: Vec<NamedEntity>,
    pub base_mcp_server_id: Option<String>,
    pub is_system: bool,
    pub total_uses: i64,
    pub session_count: i64,
    pub avg_effectiveness: String,
    pub scored_sessions: i64,
    pub goal_achievement_pct: String,
}

#[derive(Debug, Clone, Serialize, Copy)]
pub struct McpServerStats {
    pub total_count: usize,
    pub enabled_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MyMcpServersPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub servers: Vec<McpServerView>,
    pub stats: McpServerStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct SecretVarView {
    pub id: String,
    pub plugin_id: String,
    pub var_name: String,
    pub var_value: String,
    pub is_secret: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SecretGroupView {
    pub plugin_id: String,
    pub variables: Vec<SecretVarView>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Copy)]
pub struct SecretsStats {
    pub total_count: usize,
    pub secret_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MySecretsPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub groups: Vec<SecretGroupView>,
    pub plugins: Vec<NamedEntity>,
    pub stats: SecretsStats,
}
