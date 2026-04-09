use systemprompt::database::DbPool;
use systemprompt::generator::prerender_content;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::MarketplaceError;

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

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let start = std::time::Instant::now();

        tracing::info!("Content prerender started");

        let db_pool = ctx.db_pool::<DbPool>().ok_or(MarketplaceError::Internal(
            "Database not available in job context".to_string(),
        ))?;

        prerender_content(db_pool.clone()).await?;

        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::info!(duration_ms, "Content prerender completed");

        Ok(JobResult::success().with_duration(duration_ms))
    }
}

systemprompt::traits::submit_job!(&ContentPrerenderJob);
