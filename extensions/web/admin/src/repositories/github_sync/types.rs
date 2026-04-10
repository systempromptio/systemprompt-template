#[derive(Debug, Clone)]
pub struct SyncResult {
    pub commit_hash: String,
    pub plugins_synced: u64,
    pub errors: u64,
    pub changed: bool,
    pub duration_ms: u64,
}

pub(super) struct PluginImportTally {
    pub plugin_ids: Vec<String>,
    pub success_count: u64,
    pub error_count: u64,
}
