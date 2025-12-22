//! Content ingestion background job.
//!
//! This job demonstrates how extensions integrate with core's scheduler:
//!
//! 1. Implement the `Job` trait from `systemprompt_traits`
//! 2. Register with `submit_job!()` macro for scheduler discovery
//! 3. Access database via `JobContext::db_pool()`
//!
//! The job is automatically discovered at compile time via the `inventory` crate.

use std::sync::Arc;

use anyhow::Result;
use sqlx::PgPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::config::BlogConfig;
use crate::services::IngestionService;

/// Scheduled job that ingests markdown content from configured directories.
///
/// # Integration with Core Scheduler
///
/// This job is registered with core's scheduler via the `submit_job!` macro
/// at the bottom of this file. The scheduler discovers it at compile time
/// using the `inventory` crate and runs it according to the cron schedule.
///
/// # Usage
///
/// The job can be triggered in two ways:
///
/// 1. **Scheduled**: Runs automatically every hour via the scheduler
/// 2. **Direct**: Call `execute_with_pool()` for manual triggering
///
/// # Example
///
/// ```rust,ignore
/// // Direct execution (e.g., at server startup)
/// let result = ContentIngestionJob::execute_with_pool(pool, &config).await?;
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ContentIngestionJob;

impl ContentIngestionJob {
    /// Execute ingestion with a database pool (for direct calls).
    ///
    /// This method allows the job to be triggered directly without going
    /// through the scheduler, useful for:
    /// - Running at server startup
    /// - Manual triggering via API
    /// - Testing
    pub async fn execute_with_pool(pool: Arc<PgPool>, config: &BlogConfig) -> Result<JobResult> {
        let start = std::time::Instant::now();

        tracing::info!("Blog content ingestion started");

        let ingestion_service = IngestionService::new(pool);

        let mut total_processed = 0u64;
        let mut total_errors = 0u64;

        for source in &config.content_sources {
            if !source.enabled {
                tracing::debug!(source = %source.source_id, "Skipping disabled source");
                continue;
            }

            tracing::debug!(source = %source.source_id, path = %source.path.display(), "Ingesting source");

            match ingestion_service.ingest_source(source).await {
                Ok(report) => {
                    total_processed += report.files_processed as u64;
                    total_errors += report.errors.len() as u64;

                    for error in &report.errors {
                        tracing::warn!(source = %source.source_id, error = %error, "Ingestion warning");
                    }

                    tracing::info!(
                        source = %source.source_id,
                        files_found = report.files_found,
                        files_processed = report.files_processed,
                        errors = report.errors.len(),
                        "Source ingested"
                    );
                }
                Err(e) => {
                    tracing::error!(source = %source.source_id, error = %e, "Source ingestion failed");
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

    /// Load BlogConfig from standard location.
    ///
    /// Looks for config at:
    /// 1. `BLOG_CONFIG` environment variable
    /// 2. `./services/config/blog.yaml` (default)
    fn load_config() -> Result<BlogConfig> {
        let config_path = std::env::var("BLOG_CONFIG")
            .unwrap_or_else(|_| "./services/config/blog.yaml".to_string());

        if std::path::Path::new(&config_path).exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: BlogConfig = serde_yaml::from_str(&content)?;
            tracing::debug!(path = %config_path, sources = config.content_sources.len(), "Loaded blog config");
            Ok(config)
        } else {
            tracing::warn!(path = %config_path, "Blog config not found, using defaults");
            Ok(BlogConfig::default())
        }
    }
}

/// Job trait implementation for scheduler integration.
///
/// This allows the job to be discovered and executed by core's scheduler.
#[async_trait::async_trait]
impl Job for ContentIngestionJob {
    fn name(&self) -> &'static str {
        "blog_content_ingestion"
    }

    fn description(&self) -> &'static str {
        "Ingests markdown content from configured blog directories into the database"
    }

    fn schedule(&self) -> &'static str {
        "0 0 * * * *" // Every hour
    }

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        // Get database pool from job context
        let pool = ctx
            .db_pool::<PgPool>()
            .ok_or_else(|| anyhow::anyhow!("Database pool not available in job context"))?;

        // Load configuration
        let config = Self::load_config()?;

        // Execute ingestion
        Self::execute_with_pool(Arc::new(pool.clone()), &config).await
    }
}

// Register job with the scheduler's inventory system.
// This makes the job discoverable by core's scheduler at compile time.
systemprompt::traits::submit_job!(&ContentIngestionJob);
