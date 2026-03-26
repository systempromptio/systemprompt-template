use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSkill {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub content: String,
    pub tags: Vec<String>,
    pub base_skill_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MarketplaceVersion {
    pub id: String,
    pub user_id: String,
    pub version_number: i32,
    pub version_type: String,
    pub snapshot_path: String,
    pub skills_snapshot: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MarketplaceVersionSummary {
    pub id: String,
    pub user_id: String,
    pub version_number: i32,
    pub version_type: String,
    pub snapshot_path: String,
    pub skills_count: i32,
    pub skill_names: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MarketplaceChangelogEntry {
    pub id: String,
    pub user_id: String,
    pub version_id: Option<String>,
    pub action: String,
    pub skill_id: String,
    pub skill_name: String,
    pub detail: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SyncDiff {
    pub added: Vec<ParsedSkill>,
    pub updated: Vec<(ParsedSkill, String)>,
    pub deleted: Vec<(String, String)>, // (skill_id, skill_name)
}

#[derive(Debug, Serialize)]
pub struct MarketplaceUploadResponse {
    pub version_number: i32,
    pub skills_added: usize,
    pub skills_updated: usize,
    pub skills_deleted: usize,
    pub changelog: Vec<MarketplaceChangelogEntry>,
}

#[derive(Debug, Serialize)]
pub struct MarketplaceRestoreResponse {
    pub restored_version: i32,
    pub new_version: i32,
    pub skills_restored: usize,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AllVersionsSummaryRow {
    pub id: String,
    pub user_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub version_number: i32,
    pub version_type: String,
    pub skills_count: i32,
    pub skill_names: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NewChangelogEntry {
    pub user_id: String,
    pub version_id: String,
    pub action: String,
    pub skill_id: String,
    pub skill_name: String,
    pub detail: String,
}
