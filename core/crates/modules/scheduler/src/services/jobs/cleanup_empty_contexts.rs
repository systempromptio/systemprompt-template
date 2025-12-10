use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::AppContext;

use crate::repository::SchedulerRepository;

pub async fn cleanup_empty_contexts(
    db_pool: DbPool,
    logger: LogService,
    _app_context: Arc<AppContext>,
) -> Result<()> {
    let start_time = std::time::Instant::now();

    logger
        .info("scheduler", "Job started | job=cleanup_empty_contexts")
        .await
        .ok();

    let repository = SchedulerRepository::new(db_pool);
    let deleted_count = repository.cleanup_empty_contexts(1).await?;

    logger
        .log(
            LogLevel::Info,
            "scheduler",
            &format!(
                "Job completed | job=cleanup_empty_contexts, deleted_contexts={}",
                deleted_count
            ),
            Some(json!({
                "job_name": "cleanup_empty_contexts",
                "deleted_contexts": deleted_count,
                "duration_ms": start_time.elapsed().as_millis(),
            })),
        )
        .await
        .ok();

    Ok(())
}
