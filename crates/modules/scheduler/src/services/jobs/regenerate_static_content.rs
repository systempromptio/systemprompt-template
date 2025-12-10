use crate::services::static_content::{generate_sitemap, optimize_images, prerender_content};
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::AppContext;

pub async fn regenerate_static_content(
    db_pool: DbPool,
    logger: LogService,
    app_context: Arc<AppContext>,
) -> Result<()> {
    let start_time = std::time::Instant::now();

    logger
        .info("scheduler", "Job started | job=regenerate_static_content")
        .await
        .ok();

    // Optimize images FIRST so prerender uses optimized versions
    let mut images_success = false;
    match optimize_images(db_pool.clone(), logger.clone()).await {
        Ok(_) => {
            images_success = true;
        },
        Err(e) => {
            logger
                .warn(
                    "scheduler",
                    &format!("Image optimization failed | error={e}"),
                )
                .await
                .ok();
        },
    }

    let mut prerender_success = false;
    match prerender_content(db_pool.clone(), logger.clone(), app_context.clone()).await {
        Ok(_) => {
            prerender_success = true;
        },
        Err(e) => {
            logger
                .warn("scheduler", &format!("Prerendering failed | error={e}"))
                .await
                .ok();
        },
    }

    let mut sitemap_success = false;
    match generate_sitemap(db_pool.clone(), logger.clone(), app_context.clone()).await {
        Ok(_) => {
            sitemap_success = true;
        },
        Err(e) => {
            logger
                .warn(
                    "scheduler",
                    &format!("Sitemap generation failed | error={e}"),
                )
                .await
                .ok();
        },
    }

    logger
        .log(
            LogLevel::Info,
            "scheduler",
            &format!(
                "Job completed | job=regenerate_static_content, images={}, prerender={}, \
                 sitemap={}",
                if images_success { "success" } else { "failed" },
                if prerender_success {
                    "success"
                } else {
                    "failed"
                },
                if sitemap_success { "success" } else { "failed" },
            ),
            Some(json!({
                "job_name": "regenerate_static_content",
                "images_success": images_success,
                "prerender_success": prerender_success,
                "sitemap_success": sitemap_success,
                "duration_ms": start_time.elapsed().as_millis(),
            })),
        )
        .await
        .ok();

    Ok(())
}
