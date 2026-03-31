use anyhow::Result;
use systemprompt::database::DbPool;
use systemprompt::generator::generate_sitemap;
use systemprompt::traits::{Job, JobContext, JobResult};

#[derive(Debug, Clone, Copy, Default)]
pub struct SitemapGenerationJob;

#[async_trait::async_trait]
impl Job for SitemapGenerationJob {
    fn name(&self) -> &'static str {
        "sitemap_generation"
    }

    fn description(&self) -> &'static str {
        "Generates sitemap.xml from all published content"
    }

    fn schedule(&self) -> &'static str {
        "0 0 * * * *"
    }

    fn run_on_startup(&self) -> bool {
        false
    }

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        let start = std::time::Instant::now();

        tracing::info!("Sitemap generation started");

        let db_pool = ctx
            .db_pool::<DbPool>()
            .ok_or_else(|| anyhow::anyhow!("Database not available in job context"))?;

        generate_sitemap(db_pool.clone()).await?;

        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::info!(duration_ms, "Sitemap generation completed");

        Ok(JobResult::success().with_duration(duration_ms))
    }
}

systemprompt::traits::submit_job!(&SitemapGenerationJob);
