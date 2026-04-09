use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::config::BlogConfigValidated;
use crate::error::MarketplaceError;
use crate::models::IngestionOptions;
use crate::services::IngestionService;

#[derive(Debug, Clone, Copy, Default)]
pub struct ContentIngestionJob;

impl ContentIngestionJob {
    pub async fn execute_with_config(
        pool: Arc<PgPool>,
        config: &BlogConfigValidated,
    ) -> anyhow::Result<JobResult> {
        Self::execute_with_options(pool, config, IngestionOptions::default()).await
    }

    pub async fn execute_with_options(
        pool: Arc<PgPool>,
        config: &BlogConfigValidated,
        options: IngestionOptions,
    ) -> anyhow::Result<JobResult> {
        let start = std::time::Instant::now();

        tracing::info!(
            delete_orphans = options.delete_orphans,
            "Blog content ingestion started"
        );

        let ingestion_service = IngestionService::new(pool);

        let mut total_processed = 0u64;
        let mut total_errors = 0u64;
        let mut total_orphans_deleted = 0u64;

        for source in config.enabled_sources() {
            tracing::debug!(
                source_id = %source.source_id(),
                path = %source.path().display(),
                "Ingesting source"
            );

            match ingestion_service
                .ingest_path_with_options(
                    source.path(),
                    source.source_id(),
                    source.category_id(),
                    options,
                )
                .await
            {
                Ok(report) => {
                    total_processed += report.files_processed as u64;
                    total_errors += report.errors.len() as u64;
                    total_orphans_deleted += report.orphans_deleted as u64;

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
                        orphans_deleted = report.orphans_deleted,
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

        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::info!(
            files_processed = total_processed,
            orphans_deleted = total_orphans_deleted,
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
        "Ingests markdown content from configured blog directories into the database. Set CONTENT_INGESTION_DELETE_ORPHANS=true to clean up orphaned records."
    }

    fn schedule(&self) -> &'static str {
        "0 0 * * * *"
    }

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let db = ctx.db_pool::<DbPool>().ok_or(MarketplaceError::Internal(
            "Database not available in job context".to_string(),
        ))?;

        let pool = db.write_pool().ok_or(MarketplaceError::Internal(
            "Write PgPool not available from database".to_string(),
        ))?;

        let config = BlogConfigValidated::load_from_env_or_default()
            .map_err(|e| MarketplaceError::Internal(format!("Failed to load blog config: {e}")))?;

        let delete_orphans = std::env::var("CONTENT_INGESTION_DELETE_ORPHANS")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false);

        let options = IngestionOptions::default().with_delete_orphans(delete_orphans);

        Self::execute_with_options(pool, &config, options).await
    }
}

systemprompt::traits::submit_job!(&ContentIngestionJob);
