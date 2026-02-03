use anyhow::Result;
use systemprompt::database::DbPool;
use systemprompt::generator::prerender_content;
use systemprompt::traits::{Job, JobContext, JobResult};

#[derive(Debug, Clone, Copy, Default)]
pub struct ContentPrerenderJob;

#[async_trait::async_trait]
impl Job for ContentPrerenderJob {
    fn name(&self) -> &'static str {
        "content_prerender"
    }

    fn description(&self) -> &'static str {
        "Prerenders static HTML pages with enriched content from database"
    }

    fn schedule(&self) -> &'static str {
        "0 */15 * * * *"
    }

    fn run_on_startup(&self) -> bool {
        false
    }

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        let start = std::time::Instant::now();

        tracing::info!("Content prerender started");

        let db_pool = ctx
            .db_pool::<DbPool>()
            .ok_or_else(|| anyhow::anyhow!("Database not available in job context"))?;

        prerender_content(db_pool.clone()).await?;

        #[allow(clippy::cast_possible_truncation)]
        let duration_ms = start.elapsed().as_millis() as u64;

        tracing::info!(duration_ms, "Content prerender completed");

        Ok(JobResult::success().with_duration(duration_ms))
    }
}

systemprompt::traits::submit_job!(&ContentPrerenderJob);
