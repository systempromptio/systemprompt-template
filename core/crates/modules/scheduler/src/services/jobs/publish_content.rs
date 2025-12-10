use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::AppContext;

use super::content_ingestion::ingest_content;
use crate::services::static_content::{
    generate_sitemap, optimize_images, organize_css_files, prerender_content,
};

pub async fn publish_content(
    db_pool: DbPool,
    logger: LogService,
    app_context: Arc<AppContext>,
) -> Result<()> {
    let start_time = std::time::Instant::now();

    logger
        .info("publish", "Job started | job=publish_content")
        .await
        .ok();

    let mut steps_succeeded = 0;
    let mut steps_failed = 0;

    if let Err(e) = optimize_images(db_pool.clone(), logger.clone()).await {
        logger
            .warn(
                "publish",
                &format!("Image optimization warning | error={e}"),
            )
            .await
            .ok();
        steps_failed += 1;
    } else {
        steps_succeeded += 1;
    }

    ingest_content(db_pool.clone(), logger.clone(), app_context.clone()).await?;
    steps_succeeded += 1;

    tokio::time::sleep(Duration::from_millis(500)).await;

    match prerender_content(db_pool.clone(), logger.clone(), app_context.clone()).await {
        Ok(_) => {
            logger
                .debug("publish", "Prerendering completed | status=success")
                .await
                .ok();
            steps_succeeded += 1;
        },
        Err(e) => {
            logger
                .warn("publish", &format!("Prerendering warning | error={e}"))
                .await
                .ok();
            steps_failed += 1;
        },
    }

    match generate_sitemap(db_pool.clone(), logger.clone(), app_context.clone()).await {
        Ok(_) => {
            logger
                .debug("publish", "Sitemap generated | status=success")
                .await
                .ok();
            steps_succeeded += 1;
        },
        Err(e) => {
            logger
                .warn(
                    "publish",
                    &format!("Sitemap generation warning | error={e}"),
                )
                .await
                .ok();
            steps_failed += 1;
        },
    }

    let web_dir = std::env::var("WEB_DIR").unwrap_or_else(|_| "/app/core/web/dist".to_string());
    match organize_css_files(&web_dir).await {
        Ok(count) => {
            logger
                .debug("publish", &format!("CSS organized | files={count}"))
                .await
                .ok();
            steps_succeeded += 1;
        },
        Err(e) => {
            logger
                .warn(
                    "publish",
                    &format!("CSS organization warning | error={e}"),
                )
                .await
                .ok();
            steps_failed += 1;
        },
    }

    logger
        .log(
            LogLevel::Info,
            "scheduler",
            "Job completed | job=publish_content",
            Some(json!({
                "job_name": "publish_content",
                "steps_succeeded": steps_succeeded,
                "steps_failed": steps_failed,
                "total_steps": steps_succeeded + steps_failed,
                "duration_ms": start_time.elapsed().as_millis(),
            })),
        )
        .await
        .ok();

    Ok(())
}
