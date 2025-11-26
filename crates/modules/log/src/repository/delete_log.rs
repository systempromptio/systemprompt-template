use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, ToDbValue};

pub async fn delete_log_entry(db: &dyn DatabaseProvider, id: &str) -> Result<bool> {
    let query = DatabaseQueryEnum::DeleteLog.get(db);
    let rows_affected = db
        .execute(&query, &[&id])
        .await
        .context("Failed to delete log entry")?;

    Ok(rows_affected > 0)
}

pub async fn delete_log_entries(db: &dyn DatabaseProvider, ids: &[String]) -> Result<u64> {
    if ids.is_empty() {
        return Ok(0);
    }

    let placeholders = ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("${}", i + 1))
        .collect::<Vec<_>>()
        .join(",");
    let query_str = format!("DELETE FROM logs WHERE id IN ({placeholders})");
    let params: Vec<&dyn ToDbValue> = ids.iter().map(|id| -> &dyn ToDbValue { id }).collect();
    let rows_affected = db
        .execute(&query_str, &params)
        .await
        .context("Failed to delete multiple log entries")?;
    Ok(rows_affected)
}

pub async fn cleanup_old_logs(db: &dyn DatabaseProvider, older_than: DateTime<Utc>) -> Result<u64> {
    let query = DatabaseQueryEnum::DeleteOldLogs.get(db);
    let rows_affected = db
        .execute(&query, &[&older_than])
        .await
        .context("Failed to cleanup old logs")?;

    Ok(rows_affected)
}

pub async fn clear_all_logs(db: &dyn DatabaseProvider) -> Result<u64> {
    let clear_query = "DELETE FROM logs";
    let rows_affected = db
        .execute(&clear_query, &[])
        .await
        .context("Failed to clear all logs")?;

    Ok(rows_affected)
}
