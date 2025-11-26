use anyhow::{Context, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};

use crate::models::LogEntry;

pub async fn create_log(db: &dyn DatabaseProvider, entry: &LogEntry) -> Result<()> {
    let metadata_json = entry
        .metadata
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .context("Failed to serialize log metadata")?;

    let level_str = entry.level.to_string();
    let query = DatabaseQueryEnum::CreateLog.get(db);

    db.execute(
        &query,
        &[
            &level_str,
            &entry.module,
            &entry.message,
            &metadata_json,
            &entry.user_id,
            &entry.session_id,
            &entry.task_id,
            &entry.trace_id,
            &entry.context_id,
            &entry.client_id,
        ],
    )
    .await
    .context("Failed to create log entry")?;

    Ok(())
}
