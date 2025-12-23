use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::types::SyncDirection;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SyncFilesResult {
    pub direction: SyncDirection,
    pub dry_run: bool,
    pub files_synced: Vec<SyncedFile>,
    pub files_skipped: Vec<SkippedFile>,
    pub summary: SyncSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SyncedFile {
    pub path: String,
    pub action: SyncAction,
    pub size_bytes: u64,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SkippedFile {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SyncAction {
    Created,
    Updated,
    Deleted,
    Unchanged,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct SyncSummary {
    pub total_files: usize,
    pub created: usize,
    pub updated: usize,
    pub deleted: usize,
    pub unchanged: usize,
    pub skipped: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SyncDatabaseResult {
    pub direction: SyncDirection,
    pub dry_run: bool,
    pub tables_synced: Vec<TableSyncResult>,
    pub summary: DatabaseSyncSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TableSyncResult {
    pub table_name: String,
    pub records_synced: usize,
    pub records_created: usize,
    pub records_updated: usize,
    pub records_deleted: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct DatabaseSyncSummary {
    pub total_tables: usize,
    pub total_records_synced: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeployCrateResult {
    pub success: bool,
    pub image_tag: String,
    pub build_skipped: bool,
    pub steps_completed: Vec<DeployStep>,
    pub deployment_url: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeployStep {
    pub name: String,
    pub status: StepStatus,
    pub message: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    Success,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SyncAllResult {
    pub direction: SyncDirection,
    pub dry_run: bool,
    pub files_result: Option<SyncFilesResult>,
    pub database_result: Option<SyncDatabaseResult>,
    pub deploy_result: Option<DeployCrateResult>,
    pub total_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SyncStatusResult {
    pub tenant_id: String,
    pub api_url: String,
    pub services_path: String,
    pub database_configured: bool,
    pub cloud_status: CloudStatus,
    pub last_sync: Option<LastSyncInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CloudStatus {
    pub connected: bool,
    pub deployment_status: Option<String>,
    pub last_deployment: Option<DateTime<Utc>>,
    pub app_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LastSyncInfo {
    pub direction: SyncDirection,
    pub timestamp: DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SyncTable {
    Agents,
    Skills,
    Contexts,
}

impl std::fmt::Display for SyncTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Agents => write!(f, "agents"),
            Self::Skills => write!(f, "skills"),
            Self::Contexts => write!(f, "contexts"),
        }
    }
}

impl std::str::FromStr for SyncTable {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "agents" => Ok(Self::Agents),
            "skills" => Ok(Self::Skills),
            "contexts" => Ok(Self::Contexts),
            _ => anyhow::bail!("Invalid sync table: {}", s),
        }
    }
}
