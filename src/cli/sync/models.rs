use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    ToDisk,
    ToDatabase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DiffStatus {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContentDiffItem {
    pub slug: String,
    pub source_id: String,
    pub status: DiffStatus,
    pub disk_hash: Option<String>,
    pub db_hash: Option<String>,
    pub disk_updated_at: Option<DateTime<Utc>>,
    pub db_updated_at: Option<DateTime<Utc>>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillDiffItem {
    pub skill_id: String,
    pub file_path: String,
    pub status: DiffStatus,
    pub disk_hash: Option<String>,
    pub db_hash: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct ContentDiffResult {
    pub source_id: String,
    pub added: Vec<ContentDiffItem>,
    pub removed: Vec<ContentDiffItem>,
    pub modified: Vec<ContentDiffItem>,
    pub unchanged: usize,
}

impl ContentDiffResult {
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty() || !self.modified.is_empty()
    }
}

#[derive(Debug, Default, Serialize)]
pub struct SkillsDiffResult {
    pub added: Vec<SkillDiffItem>,
    pub removed: Vec<SkillDiffItem>,
    pub modified: Vec<SkillDiffItem>,
    pub unchanged: usize,
}

impl SkillsDiffResult {
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty() || !self.modified.is_empty()
    }
}

#[derive(Debug, Default, Serialize)]
pub struct SyncResult {
    pub items_synced: usize,
    pub items_skipped: usize,
    pub items_deleted: usize,
    pub errors: Vec<String>,
    pub direction: String,
}

pub struct DiskContent {
    pub slug: String,
    pub title: String,
    pub body: String,
}

pub struct DiskSkill {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub file_path: String,
}
