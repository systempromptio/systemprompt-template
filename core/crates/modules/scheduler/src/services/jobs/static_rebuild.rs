use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;
use tokio::process::Command;

pub async fn rebuild_static_site(
    _db_pool: DbPool,
    logger: LogService,
    _app_context: Arc<AppContext>,
) -> Result<()> {
    logger
        .info("scheduler", "Job started | job=static_rebuild")
        .await
        .ok();

    let enabled = std::env::var("STATIC_REBUILD_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    if !enabled {
        logger
            .debug(
                "scheduler",
                "Job skipped | job=static_rebuild, enabled=false",
            )
            .await
            .ok();
        return Ok(());
    }

    let web_dir = std::env::var("WEB_DIR").unwrap_or_else(|_| "core/web".to_string());

    let start = std::time::Instant::now();

    let output = Command::new("npm")
        .current_dir(&web_dir)
        .args(&["run", "build:full"])
        .output()
        .await;

    match output {
        Ok(result) => {
            let duration = start.elapsed().as_secs();
            if result.status.success() {
                logger
                    .info(
                        "scheduler",
                        &format!(
                            "Job completed | job=static_rebuild, duration_secs={}",
                            duration
                        ),
                    )
                    .await
                    .ok();
            } else {
                let err_msg = String::from_utf8_lossy(&result.stderr);
                logger
                    .error(
                        "scheduler",
                        &format!("Job failed | job=static_rebuild, error={err_msg}"),
                    )
                    .await
                    .ok();
            }
        },
        Err(e) => {
            logger
                .error(
                    "scheduler",
                    &format!("Job failed | job=static_rebuild, error={e}"),
                )
                .await
                .ok();
        },
    }

    Ok(())
}
