use anyhow::{Context, Result};
use systemprompt_core_database::DatabaseProvider;

use crate::models::LogEntry;

pub async fn update_log_entry(
    db: &dyn DatabaseProvider,
    id: &str,
    entry: &LogEntry,
) -> Result<bool> {
    let metadata_json = entry
        .metadata
        .as_ref()
        .map(|m| serde_json::to_string(m).unwrap_or_default());

    let level_str = entry.level.to_string();

    let update_query =
        "UPDATE logs SET level = ?, module = ?, message = ?, metadata = ? WHERE id = ?";
    let rows_affected = db
        .execute(
            &update_query,
            &[
                &level_str,
                &entry.module,
                &entry.message,
                &metadata_json,
                &id,
            ],
        )
        .await
        .context("Failed to update log entry")?;

    Ok(rows_affected > 0)
}
