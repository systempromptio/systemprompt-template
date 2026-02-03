use std::sync::Arc;

use anyhow::Result;
use systemprompt::database::DbPool;
use systemprompt::generator::{generate_feed, organize_dist_assets, prerender_pages};
use systemprompt::models::AppPaths;
use systemprompt::traits::{Job, JobContext, JobResult};

use super::{
    ContentIngestionJob, ContentPrerenderJob, CopyExtensionAssetsJob, LlmsTxtGenerationJob,
    RobotsTxtGenerationJob, SitemapGenerationJob,
};

#[derive(Default)]
struct PipelineStats {
    succeeded: u64,
    failed: u64,
}

impl PipelineStats {
    fn record_success(&mut self) {
        self.succeeded += 1;
    }

    fn record_failure(&mut self) {
        self.failed += 1;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PublishPipelineJob;

impl PublishPipelineJob {
    async fn run_ingestion(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        match ContentIngestionJob.execute(ctx).await {
            Ok(result) => {
                tracing::debug!(
                    processed = result.items_processed.unwrap_or(0),
                    errors = result.items_failed.unwrap_or(0),
                    "Content ingestion completed"
                );
                stats.record_success();
            }
            Err(e) => {
                tracing::error!(error = %e, "Content ingestion failed");
                stats.record_failure();
            }
        }
    }

    async fn run_asset_copy(&self, stats: &mut PipelineStats) {
        match CopyExtensionAssetsJob::execute_copy().await {
            Ok(result) => {
                tracing::debug!(
                    copied = result.items_processed.unwrap_or(0),
                    failed = result.items_failed.unwrap_or(0),
                    "Asset copy completed"
                );
                stats.record_success();
            }
            Err(e) => {
                tracing::error!(error = %e, "Asset copy failed");
                stats.record_failure();
            }
        }
    }

    async fn run_prerender(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        match ContentPrerenderJob.execute(ctx).await {
            Ok(_result) => {
                tracing::debug!("Content prerender completed");
                stats.record_success();
            }
            Err(e) => {
                tracing::error!(error = %e, "Content prerender failed");
                stats.record_failure();
            }
        }
    }

    async fn run_page_prerender(&self, db_pool: &DbPool, stats: &mut PipelineStats) {
        match prerender_pages(Arc::clone(db_pool)).await {
            Ok(results) => {
                tracing::debug!(page_count = results.len(), "Page prerendering completed");
                stats.record_success();
            }
            Err(e) => {
                tracing::error!(error = %e, "Page prerendering failed");
                stats.record_failure();
            }
        }
    }

    async fn run_sitemap(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        match SitemapGenerationJob.execute(ctx).await {
            Ok(_result) => {
                tracing::debug!("Sitemap generation completed");
                stats.record_success();
            }
            Err(e) => {
                tracing::error!(error = %e, "Sitemap generation failed");
                stats.record_failure();
            }
        }
    }

    async fn run_llms_txt(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        match LlmsTxtGenerationJob.execute(ctx).await {
            Ok(_result) => {
                tracing::debug!("llms.txt generation completed");
                stats.record_success();
            }
            Err(e) => {
                tracing::error!(error = %e, "llms.txt generation failed");
                stats.record_failure();
            }
        }
    }

    async fn run_feed(&self, db_pool: &DbPool, stats: &mut PipelineStats) {
        match generate_feed(Arc::clone(db_pool)).await {
            Ok(()) => {
                tracing::debug!("RSS feed generation completed");
                stats.record_success();
            }
            Err(e) => {
                tracing::error!(error = %e, "RSS feed generation failed");
                stats.record_failure();
            }
        }
    }

    async fn run_robots_txt(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        match RobotsTxtGenerationJob.execute(ctx).await {
            Ok(_result) => {
                tracing::debug!("robots.txt generation completed");
                stats.record_success();
            }
            Err(e) => {
                tracing::error!(error = %e, "robots.txt generation failed");
                stats.record_failure();
            }
        }
    }

    async fn run_asset_organization(&self, stats: &mut PipelineStats) {
        if let Ok(paths) = AppPaths::get() {
            let dist_dir = paths.web().dist().to_path_buf();
            match organize_dist_assets(&dist_dir).await {
                Ok((css_count, js_count)) => {
                    tracing::debug!(css = css_count, js = js_count, "Assets organized");
                    stats.record_success();
                }
                Err(e) => {
                    tracing::error!(error = %e, "Asset organization failed");
                    stats.record_failure();
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl Job for PublishPipelineJob {
    fn name(&self) -> &'static str {
        "publish_pipeline"
    }

    fn description(&self) -> &'static str {
        "Full content publishing pipeline: ingestion, assets, prerender, pages, sitemap, robots.txt, RSS, llms.txt"
    }

    fn schedule(&self) -> &'static str {
        "0 */15 * * * *"
    }

    fn run_on_startup(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        let start_time = std::time::Instant::now();

        let db_pool = ctx
            .db_pool::<DbPool>()
            .ok_or_else(|| anyhow::anyhow!("Database not available in job context"))?;

        tracing::info!("Publish pipeline started");

        let mut stats = PipelineStats::default();

        self.run_ingestion(ctx, &mut stats).await;
        self.run_asset_copy(&mut stats).await;
        self.run_prerender(ctx, &mut stats).await;
        self.run_page_prerender(db_pool, &mut stats).await;
        self.run_sitemap(ctx, &mut stats).await;
        self.run_llms_txt(ctx, &mut stats).await;
        self.run_robots_txt(ctx, &mut stats).await;
        self.run_feed(db_pool, &mut stats).await;
        self.run_asset_organization(&mut stats).await;

        #[allow(clippy::cast_possible_truncation)]
        let duration_ms = start_time.elapsed().as_millis() as u64;

        tracing::info!(
            steps_succeeded = stats.succeeded,
            steps_failed = stats.failed,
            duration_ms,
            "Publish pipeline completed"
        );

        Ok(JobResult::success()
            .with_stats(stats.succeeded, stats.failed)
            .with_duration(duration_ms))
    }
}

systemprompt::traits::submit_job!(&PublishPipelineJob);
