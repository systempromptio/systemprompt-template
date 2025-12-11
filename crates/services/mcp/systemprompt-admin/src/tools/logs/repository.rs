use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;

use super::models::{LogEntry, LogStats};

pub struct LogsRepository {
    pool: Arc<PgPool>,
}

impl LogsRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }

    pub async fn fetch_recent_logs(
        &self,
        page: i32,
        limit: i32,
        level: Option<&str>,
    ) -> Result<Vec<LogEntry>> {
        let offset = page * limit;

        let rows = sqlx::query!(
            r#"
            SELECT
                id,
                timestamp::text as timestamp,
                level,
                module,
                message,
                user_id,
                session_id,
                context_id
            FROM logs
            WHERE ($1::text IS NULL OR UPPER(level) = UPPER($1))
            ORDER BY timestamp DESC
            LIMIT $2 OFFSET $3
            "#,
            level,
            limit as i64,
            offset as i64
        )
        .fetch_all(&*self.pool)
        .await?;

        let entries = rows
            .into_iter()
            .map(|row| LogEntry {
                id: row.id,
                timestamp: row.timestamp.unwrap_or_default(),
                level: row.level,
                module: row.module,
                message: row.message,
                user_id: row.user_id,
                session_id: row.session_id,
                context_id: row.context_id,
            })
            .collect();

        Ok(entries)
    }

    pub async fn fetch_log_stats(&self) -> Result<LogStats> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as total_logs,
                COUNT(*) FILTER (WHERE UPPER(level) = 'ERROR') as error_count,
                COUNT(*) FILTER (WHERE UPPER(level) = 'WARN') as warn_count,
                COUNT(*) FILTER (WHERE UPPER(level) = 'INFO') as info_count,
                COUNT(DISTINCT module) as unique_modules,
                COUNT(DISTINCT user_id) as unique_users,
                MAX(timestamp)::text as last_log_time
            FROM logs
            "#
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(LogStats {
            total_logs: row.total_logs.unwrap_or(0),
            error_count: row.error_count.unwrap_or(0),
            warn_count: row.warn_count.unwrap_or(0),
            info_count: row.info_count.unwrap_or(0),
            unique_modules: row.unique_modules.unwrap_or(0),
            unique_users: row.unique_users.unwrap_or(0),
            last_log_time: row.last_log_time,
        })
    }
}
