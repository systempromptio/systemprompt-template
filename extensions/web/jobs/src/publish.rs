//! `publish_pipeline` job: the ordered composite every other job runs under.
//!
//! Order is load-bearing — bundles must exist before assets are copied, and
//! content must be ingested before it can be prerendered. A failing sub-job is
//! recorded and the pipeline continues, so one broken stage does not block the
//! rest. The sitemap and RSS feed are generated after prerender from the
//! content sources' `sitemap` and `rss` blocks in the published `ConfigView`.

use std::sync::Arc;

use systemprompt::config::AppPaths;
use systemprompt::content::ContentHooks;
use systemprompt::database::DbPool;
use systemprompt::generator::{
    PrerenderEnv, PrerenderSettings, generate_feed, generate_sitemap, organize_dist_assets,
    prerender_content, prerender_pages,
};
use systemprompt::manifest::ConfigView;
use systemprompt::traits::{Job, JobContext, JobResult};

use systemprompt_web_shared::asset_jobs::{
    AssetDeclaration, BundleAdminCssJob, CopyExtensionAssetsJob,
};

use super::{ContentIngestionJob, LlmsTxtGenerationJob, RobotsTxtGenerationJob};
use crate::error::JobError;

#[doc(hidden)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PipelineStats {
    pub succeeded: u64,
    pub failed: u64,
}

impl PipelineStats {
    pub const fn record_success(&mut self) {
        self.succeeded += 1;
    }

    pub const fn record_failure(&mut self) {
        self.failed += 1;
    }
}

/// The runtime configuration the sitemap step reads. `ConfigView` is an
/// install seed that job contexts do not carry, so the web member provides
/// it, sealed, to its own jobs.
#[derive(Debug, Clone)]
pub struct PublishConfig(pub Arc<ConfigView>);

#[derive(Debug, Clone, Copy)]
pub struct PublishPipelineJob {
    assets: AssetDeclaration,
}

impl PublishPipelineJob {
    #[must_use]
    pub const fn new(assets: AssetDeclaration) -> Self {
        Self { assets }
    }

    async fn run_ingestion(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        match ContentIngestionJob.execute(ctx).await {
            Ok(result) => {
                tracing::debug!(
                    processed = result.items_processed.unwrap_or(0),
                    errors = result.items_failed.unwrap_or(0),
                    "Content ingestion completed"
                );
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "Content ingestion failed");
                stats.record_failure();
            },
        }
    }

    async fn run_bundle_admin_css(&self, paths: &AppPaths, stats: &mut PipelineStats) {
        match BundleAdminCssJob::execute_bundle(paths).await {
            Ok(result) => {
                tracing::debug!(
                    bundled = result.items_processed.unwrap_or(0),
                    "Admin CSS bundle completed"
                );
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "Admin CSS bundle failed");
                stats.record_failure();
            },
        }
    }

    async fn run_asset_copy(&self, paths: &AppPaths, stats: &mut PipelineStats) {
        match CopyExtensionAssetsJob::execute_copy(paths, (self.assets)(paths)).await {
            Ok(result) => {
                tracing::debug!(
                    copied = result.items_processed.unwrap_or(0),
                    failed = result.items_failed.unwrap_or(0),
                    "Asset copy completed"
                );
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "Asset copy failed");
                stats.record_failure();
            },
        }
    }

    async fn run_prerender(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        let outcome = async {
            let db_pool = Arc::clone(ctx.get::<DbPool>()?);
            let paths = ctx.get::<Arc<AppPaths>>()?.as_ref();
            let settings = ctx.get::<PrerenderSettings>()?;
            let hooks = Arc::clone(ctx.get::<Arc<ContentHooks>>()?);
            let content_repo = systemprompt::content::ContentRepository::new(&db_pool);
            let env = PrerenderEnv {
                settings,
                paths,
                hooks: &hooks,
                state: ctx.dependencies(),
            };
            prerender_content(Arc::clone(&db_pool), content_repo, env)
                .await
                .map_err(|e| JobError::other(e.to_string()))?;
            let content_repo = systemprompt::content::ContentRepository::new(&db_pool);
            let env = PrerenderEnv {
                settings,
                paths,
                hooks: &hooks,
                state: ctx.dependencies(),
            };
            prerender_pages(db_pool, content_repo, env)
                .await
                .map_err(|e| JobError::other(e.to_string()))
        }
        .await;
        match outcome {
            Ok(results) => {
                tracing::debug!(
                    page_count = results.len(),
                    "Content and page prerender completed"
                );
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "Prerender failed");
                stats.record_failure();
            },
        }
    }

    async fn run_llms_txt(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        match LlmsTxtGenerationJob.execute(ctx).await {
            Ok(_result) => {
                tracing::debug!("llms.txt generation completed");
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "llms.txt generation failed");
                stats.record_failure();
            },
        }
    }

    async fn run_robots_txt(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        match RobotsTxtGenerationJob.execute(ctx).await {
            Ok(_result) => {
                tracing::debug!("robots.txt generation completed");
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "robots.txt generation failed");
                stats.record_failure();
            },
        }
    }

    async fn run_sitemap(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        let outcome = async {
            let db_pool = ctx.get::<DbPool>()?;
            let paths = ctx.get::<Arc<AppPaths>>()?.as_ref();
            let config = ctx.get::<PublishConfig>()?;
            let content_repo = systemprompt::content::ContentRepository::new(db_pool);
            generate_sitemap(content_repo, &config.0, paths).await?;
            Ok::<(), JobError>(())
        }
        .await;
        match outcome {
            Ok(()) => stats.record_success(),
            Err(e) => {
                tracing::error!(error = %e, "Sitemap generation failed");
                stats.record_failure();
            },
        }
    }

    async fn run_feed(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        let outcome = async {
            let db_pool = ctx.get::<DbPool>()?;
            let paths = ctx.get::<Arc<AppPaths>>()?.as_ref();
            let config = ctx.get::<PublishConfig>()?;
            let content_repo = systemprompt::content::ContentRepository::new(db_pool);
            generate_feed(content_repo, &config.0, paths).await?;
            Ok::<(), JobError>(())
        }
        .await;
        match outcome {
            Ok(()) => stats.record_success(),
            Err(e) => {
                tracing::error!(error = %e, "RSS feed generation failed");
                stats.record_failure();
            },
        }
    }

    async fn run_derived_files(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        self.run_sitemap(ctx, stats).await;
        self.run_feed(ctx, stats).await;
        self.run_llms_txt(ctx, stats).await;
        self.run_robots_txt(ctx, stats).await;
    }

    async fn run_asset_organization(&self, paths: &AppPaths, stats: &mut PipelineStats) {
        let dist_dir = paths.web().dist().to_path_buf();
        match organize_dist_assets(&dist_dir).await {
            Ok((css_count, js_count)) => {
                tracing::debug!(css = css_count, js = js_count, "Assets organized");
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "Asset organization failed");
                stats.record_failure();
            },
        }
    }
}

#[async_trait::async_trait]
impl Job for PublishPipelineJob {
    fn name(&self) -> &'static str {
        "publish_pipeline"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![systemprompt_web_shared::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Content publishing pipeline: ingestion, assets, prerender, pages, sitemap, RSS, robots.txt, llms.txt"
    }

    fn schedule(&self) -> &'static str {
        "0 */15 * * * *"
    }
    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        Ok(self.execute_inner(ctx).await?)
    }
}

impl PublishPipelineJob {
    async fn execute_inner(&self, ctx: &JobContext) -> Result<JobResult, JobError> {
        let start_time = std::time::Instant::now();

        let paths = ctx.get::<Arc<AppPaths>>()?.as_ref();

        tracing::info!(actor = %ctx.actor().user_id.as_str(), "Publish pipeline started");

        let mut stats = PipelineStats::default();

        self.run_ingestion(ctx, &mut stats).await;
        self.run_bundle_admin_css(paths, &mut stats).await;
        self.run_asset_copy(paths, &mut stats).await;
        self.run_prerender(ctx, &mut stats).await;
        self.run_derived_files(ctx, &mut stats).await;
        self.run_asset_organization(paths, &mut stats).await;

        let duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);

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
