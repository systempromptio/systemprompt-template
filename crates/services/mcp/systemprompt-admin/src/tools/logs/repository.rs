use anyhow::Result;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbPool};

use super::models::{LogEntry, LogStats};

pub struct LogsRepository {
    pool: DbPool,
}

impl LogsRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn fetch_recent_logs(&self, page: i32, limit: i32) -> Result<Vec<LogEntry>> {
        let offset = page * limit;

        let query = DatabaseQueryEnum::ListLogsPaginated.get(self.pool.as_ref());
        let none_str: Option<String> = None;
        let logs = self
            .pool
            .fetch_all(&query, &[&none_str, &none_str, &none_str, &limit, &offset])
            .await?;

        let entries = logs
            .iter()
            .map(|row| LogEntry {
                id: row
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                timestamp: row
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                level: row
                    .get("level")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                module: row
                    .get("module")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                message: row
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                user_id: row
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                session_id: row
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                context_id: row
                    .get("context_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
            })
            .collect();

        Ok(entries)
    }

    pub async fn fetch_log_stats(&self) -> Result<LogStats> {
        let query = DatabaseQueryEnum::GetLogStats.get(self.pool.as_ref());
        let row = self.pool.fetch_one(&query, &[]).await?;

        Ok(LogStats {
            total_logs: row.get("total_logs").and_then(|v| v.as_i64()).unwrap_or(0),
            error_count: row.get("error_count").and_then(|v| v.as_i64()).unwrap_or(0),
            warn_count: row.get("warn_count").and_then(|v| v.as_i64()).unwrap_or(0),
            info_count: row.get("info_count").and_then(|v| v.as_i64()).unwrap_or(0),
            unique_modules: row
                .get("unique_modules")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            unique_users: row
                .get("unique_users")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            last_log_time: row
                .get("last_log_time")
                .and_then(|v| v.as_str())
                .map(String::from),
        })
    }
}
