use anyhow::{Context, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};

use crate::models::{LogEntry, LogRow};

pub async fn get_recent_logs(db: &dyn DatabaseProvider, limit: i64) -> Result<Vec<LogEntry>> {
    let query = DatabaseQueryEnum::ListLogs.get(db);
    let rows = db
        .fetch_all(&query, &[&limit])
        .await
        .context("Failed to get recent logs")?;

    rows.into_iter()
        .map(|r| LogRow::from_json_row(&r).map(LogEntry::from))
        .collect()
}

pub async fn get_logs_paginated(
    db: &dyn DatabaseProvider,
    page: i32,
    per_page: i32,
    level_filter: Option<&str>,
    module_filter: Option<&str>,
    message_filter: Option<&str>,
) -> Result<(Vec<LogEntry>, i64)> {
    let offset = page.saturating_sub(1).saturating_mul(per_page);
    let query = DatabaseQueryEnum::ListLogsPaginated.get(db);

    let rows = db
        .fetch_all(
            &query,
            &[
                &level_filter,
                &module_filter,
                &message_filter,
                &per_page,
                &offset,
            ],
        )
        .await
        .context("Failed to get paginated logs")?;

    let count_query = "SELECT COUNT(*) FROM logs WHERE (? IS NULL OR level = ?) AND (? IS NULL OR module = ?) AND (? IS NULL OR message LIKE ?)";
    let count_row = db
        .fetch_one(
            &count_query,
            &[
                &level_filter,
                &level_filter,
                &module_filter,
                &module_filter,
                &message_filter,
                &message_filter,
            ],
        )
        .await
        .context("Failed to count logs")?;

    let total_count = count_row
        .get("COUNT(*)")
        .or_else(|| count_row.values().next())
        .and_then(serde_json::Value::as_i64)
        .context("Failed to extract count from database result")?;

    let entries: Result<Vec<LogEntry>> = rows
        .into_iter()
        .map(|r| LogRow::from_json_row(&r).map(LogEntry::from))
        .collect();

    Ok((entries?, total_count))
}
