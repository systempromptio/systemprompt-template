//! `content_prerender` job: renders ingested content into static page
//! artifacts.

use std::sync::Arc;
use systemprompt::database::DbPool;
use systemprompt::generator::prerender_content;
use systemprompt::models::AppPaths;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;

#[derive(Debug, Clone, Copy, Default)]
pub struct ContentPrerenderJob;

#[async_trait::async_trait]
impl Job for ContentPrerenderJob {
    fn name(&self) -> &'static str {
        "content_prerender"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Prerenders static HTML pages with enriched content from database"
    }

    fn schedule(&self) -> &'static str {
        "0 */15 * * * *"
    }
    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        Ok(execute_inner(ctx).await?)
    }
}

async fn execute_inner(ctx: &JobContext) -> Result<JobResult, JobError> {
    let start = std::time::Instant::now();

    tracing::info!("Content prerender started");

    let db_pool = ctx
        .db_pool::<DbPool>()
        .ok_or(JobError::MissingContext("DbPool"))?;
    let paths = ctx
        .app_paths::<Arc<AppPaths>>()
        .ok_or(JobError::MissingContext("AppPaths"))?
        .as_ref();

    let content_repo = systemprompt::content::ContentRepository::new(db_pool)?;

    prerender_content(DbPool::clone(db_pool), content_repo, paths).await?;

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

    tracing::info!(duration_ms, "Content prerender completed");

    Ok(JobResult::success().with_duration(duration_ms))
}

systemprompt::traits::submit_job!(&ContentPrerenderJob);
