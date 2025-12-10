use crate::services::static_content::optimize_images as run_optimize_images;
use anyhow::Result;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;

pub async fn optimize_images(db_pool: DbPool, logger: LogService) -> Result<()> {
    logger
        .info("scheduler", "Job started | job=optimize_images")
        .await
        .ok();

    run_optimize_images(db_pool, logger.clone()).await?;

    logger
        .info("scheduler", "Job completed | job=optimize_images")
        .await
        .ok();

    Ok(())
}
