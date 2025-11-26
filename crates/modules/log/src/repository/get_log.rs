use anyhow::{Context, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};

use crate::models::{LogEntry, LogRow};

pub async fn get_log_by_id(db: &dyn DatabaseProvider, id: &str) -> Result<Option<LogEntry>> {
    let query = DatabaseQueryEnum::GetLog.get(db);
    let row = db
        .fetch_optional(&query, &[&id])
        .await
        .context("Failed to get log by id")?;

    row.map(|r| LogRow::from_json_row(&r).map(LogEntry::from))
        .transpose()
}
