use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrgMarketplace {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub github_repo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgMarketplaceOverview {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub plugin_count: usize,
    pub skill_count: usize,
    pub agent_count: usize,
    pub mcp_count: usize,
    pub hook_count: usize,
    pub roles: Vec<serde_json::Value>,
    pub departments: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrgMarketplaceRequest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub github_repo_url: Option<String>,
    #[serde(default)]
    pub plugin_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrgMarketplaceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub github_repo_url: Option<Option<String>>,
    pub plugin_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GitHubSyncLogEntry {
    pub id: i64,
    pub marketplace_id: String,
    pub action: String,
    pub status: String,
    pub commit_hash: Option<String>,
    pub plugin_count: i32,
    pub error_count: i32,
    pub error_message: Option<String>,
    pub triggered_by: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: DateTime<Utc>,
}
