use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbPool};
use tokio::sync::Mutex;

use crate::models::{LogEntry, LoggingError};

const BUFFER_FLUSH_SIZE: usize = 100;
const BUFFER_FLUSH_INTERVAL_SECS: u64 = 10;

#[derive(Clone)]
pub struct BufferedLogService {
    buffer: Arc<Mutex<Vec<LogEntry>>>,
    db_pool: DbPool,
}

impl std::fmt::Debug for BufferedLogService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferedLogService")
            .field("db_pool", &"DbPool")
            .finish()
    }
}

impl BufferedLogService {
    pub fn new(db_pool: DbPool) -> Self {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let buffer_clone = buffer.clone();
        let db_pool_clone = db_pool.clone();

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(BUFFER_FLUSH_INTERVAL_SECS));

            loop {
                interval.tick().await;
                if let Err(e) = Self::flush_buffer(&buffer_clone, &db_pool_clone).await {
                    eprintln!("Failed to flush log buffer: {e}");
                }
            }
        });

        Self { buffer, db_pool }
    }

    pub async fn log(&self, entry: LogEntry) -> Result<(), LoggingError> {
        entry.validate()?;

        let mut buffer = self.buffer.lock().await;
        buffer.push(entry);

        if buffer.len() >= BUFFER_FLUSH_SIZE {
            drop(buffer);
            self.flush_now().await.ok();
        }

        Ok(())
    }

    pub async fn flush_now(&self) -> Result<()> {
        Self::flush_buffer(&self.buffer, &self.db_pool).await
    }

    async fn flush_buffer(buffer: &Arc<Mutex<Vec<LogEntry>>>, db_pool: &DbPool) -> Result<()> {
        let entries = {
            let mut buf = buffer.lock().await;
            buf.drain(..).collect::<Vec<_>>()
        };

        if entries.is_empty() {
            return Ok(());
        }

        for chunk in entries.chunks(BUFFER_FLUSH_SIZE) {
            Self::batch_insert(db_pool, chunk).await?;
        }

        Ok(())
    }

    async fn batch_insert(db_pool: &DbPool, entries: &[LogEntry]) -> Result<()> {
        for entry in entries {
            let metadata_json: Option<String> = entry
                .metadata
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?;

            let level_str = entry.level.to_string();
            let query = DatabaseQueryEnum::CreateLog.get(db_pool.as_ref());

            db_pool
                .execute(
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
                .await?;
        }

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.flush_now().await?;
        Ok(())
    }
}
