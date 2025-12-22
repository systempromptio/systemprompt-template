//! Content ingestion background job.
//!
//! This job uses validated configuration from `BlogConfigValidated`:
//! - Paths are guaranteed to exist (validated at load time)
//! - IDs are typed (`SourceId`, `CategoryId`)
//! - No runtime path validation needed

use std::sync::Arc;

use anyhow::Result;
use sqlx::PgPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::config::BlogConfigValidated;
use crate::services::IngestionService;

/// Scheduled job that ingests markdown content from configured directories.
///
/// Uses `BlogConfigValidated` which guarantees:
/// - All enabled source paths exist and are directories
/// - Base URL is valid
/// - IDs are properly typed
#[derive(Debug, Clone, Copy, Default)]
pub struct ContentIngestionJob;

impl ContentIngestionJob {
    /// Execute ingestion with validated config.
    ///
    /// Paths in `config` are guaranteed to exist (validated at startup).
    pub async fn execute_with_config(
        pool: Arc<PgPool>,
        config: &BlogConfigValidated,
    ) -> Result<JobResult> {
        let start = std::time::Instant::now();

        tracing::info!("Blog content ingestion started");

        let ingestion_service = IngestionService::new(pool);

        let mut total_processed = 0u64;
        let mut total_errors = 0u64;

        for source in config.enabled_sources() {
            tracing::debug!(
                source_id = %source.source_id(),
                path = %source.path().display(),
                "Ingesting source"
            );

            match ingestion_service
                .ingest_path(source.path(), source.source_id(), source.category_id())
                .await
            {
                Ok(report) => {
                    total_processed += report.files_processed as u64;
                    total_errors += report.errors.len() as u64;

                    for error in &report.errors {
                        tracing::warn!(
                            source_id = %source.source_id(),
                            error = %error,
                            "Ingestion warning"
                        );
                    }

                    tracing::info!(
                        source_id = %source.source_id(),
                        files_found = report.files_found,
                        files_processed = report.files_processed,
                        errors = report.errors.len(),
                        "Source ingested"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        source_id = %source.source_id(),
                        error = %e,
                        "Source ingestion failed"
                    );
                    total_errors += 1;
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        tracing::info!(
            files_processed = total_processed,
            errors = total_errors,
            duration_ms = duration_ms,
            "Blog content ingestion completed"
        );

        Ok(JobResult::success()
            .with_stats(total_processed, total_errors)
            .with_duration(duration_ms))
    }
}

#[async_trait::async_trait]
impl Job for ContentIngestionJob {
    fn name(&self) -> &'static str {
        "blog_content_ingestion"
    }

    fn description(&self) -> &'static str {
        "Ingests markdown content from configured blog directories into the database"
    }

    fn schedule(&self) -> &'static str {
        "0 0 * * * *"
    }

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        let pool = ctx
            .db_pool::<PgPool>()
            .ok_or_else(|| anyhow::anyhow!("Database pool not available in job context"))?;

        let config = BlogConfigValidated::load_from_env_or_default().map_err(|e| {
            anyhow::anyhow!("Failed to load blog config: {}", e)
        })?;

        Self::execute_with_config(Arc::new(pool.clone()), &config).await
    }
}

systemprompt::traits::submit_job!(&ContentIngestionJob);
