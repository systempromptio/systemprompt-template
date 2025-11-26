use crate::services::static_content::{generate_sitemap, prerender_content};
use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;

pub async fn regenerate_static_content(
    db_pool: DbPool,
    logger: LogService,
    app_context: Arc<AppContext>,
) -> Result<()> {
    println!("\n📄 Regenerating static content...\n");
    logger
        .info(
            "content",
            "Starting static content regeneration (prerender + sitemap)",
        )
        .await
        .ok();

    println!("   1️⃣  Prerendering content pages...");
    logger
        .info("content", "Running content prerendering...")
        .await
        .ok();

    match prerender_content(db_pool.clone(), logger.clone(), app_context.clone()).await {
        Ok(_) => {
            println!("      ✅ Content prerendering completed");
            logger
                .info("content", "Content prerendering completed successfully")
                .await
                .ok();
        },
        Err(e) => {
            println!("      ⚠️  Prerendering warning: {}", e);
            logger
                .warn("content", &format!("Prerendering warning: {}", e))
                .await
                .ok();
        },
    }

    println!("   2️⃣  Generating sitemap...");
    logger.info("content", "Generating sitemap...").await.ok();

    match generate_sitemap(db_pool.clone(), logger.clone(), app_context.clone()).await {
        Ok(_) => {
            println!("      ✅ Sitemap generated");
            logger
                .info("content", "Sitemap generated successfully")
                .await
                .ok();
        },
        Err(e) => {
            println!("      ⚠️  Sitemap generation warning: {}", e);
            logger
                .warn("content", &format!("Sitemap generation warning: {}", e))
                .await
                .ok();
        },
    }

    println!("   ✨ Static content regeneration complete\n");
    logger
        .info(
            "content",
            "Static content regeneration completed successfully",
        )
        .await
        .ok();

    Ok(())
}
