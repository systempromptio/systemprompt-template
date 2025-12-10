use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use tokio::process::Command;

use super::cards::normalize_image_url;
use super::templates::{get_assets_path, load_web_config};

pub async fn optimize_images(db_pool: DbPool, logger: LogService) -> Result<()> {
    println!("\n🖼️  Optimizing images...\n");
    logger
        .info("images", "Starting image optimization")
        .await
        .ok();

    let web_config = load_web_config().await.unwrap_or_default();
    let web_public = PathBuf::from(get_assets_path(&web_config));

    let pool = db_pool.pool_arc().context("Database must be PostgreSQL")?;
    let rows = sqlx::query!(
        r#"SELECT id, image FROM markdown_content WHERE
         image IS NOT NULL AND image != '' AND
         (image_optimization_status IS NULL OR image_optimization_status != 'optimized')
         ORDER BY published_at DESC LIMIT 100"#
    )
    .fetch_all(pool.as_ref())
    .await?;

    if rows.is_empty() {
        println!("   ✅ All images already optimized");
        logger
            .info("images", "No images require optimization")
            .await
            .ok();
        return Ok(());
    }

    println!("   📋 Found {} images to optimize", rows.len());
    logger
        .debug(
            "images",
            &format!("Found {} images to optimize", rows.len()),
        )
        .await
        .ok();

    let mut optimized = 0;
    let mut skipped = 0;
    let mut errors = 0;

    for row in rows {
        let id = row.id;
        let image_path = row.image.unwrap_or_default();

        match process_image(&web_public, &image_path, &db_pool, &id, &logger).await {
            Ok(ProcessResult::Optimized) => {
                optimized += 1;
            },
            Ok(ProcessResult::Skipped(reason)) => {
                println!("   ⏭️  Skipped {}: {}", image_path, reason);
                skipped += 1;
            },
            Err(e) => {
                println!("   ❌ Failed {}: {}", image_path, e);
                logger
                    .error("images", &format!("Failed {image_path}: {e}"))
                    .await
                    .ok();
                errors += 1;
            },
        }
    }

    let summary = format!(
        "Image optimization complete: {} optimized, {} skipped, {} errors",
        optimized, skipped, errors
    );
    println!("\n   ✨ {}\n", summary);
    logger.info("images", &summary).await.ok();

    Ok(())
}

enum ProcessResult {
    Optimized,
    Skipped(String),
}

async fn process_image(
    web_public: &Path,
    image_url: &str,
    db_pool: &DbPool,
    content_id: &str,
    logger: &LogService,
) -> Result<ProcessResult> {
    let normalized_url =
        normalize_image_url(Some(image_url)).unwrap_or_else(|| image_url.to_string());

    let source_path = resolve_image_path(web_public, image_url);

    if !source_path.exists() {
        return Ok(ProcessResult::Skipped(format!(
            "Source file not found: {}",
            source_path.display()
        )));
    }

    let webp_path = resolve_image_path(web_public, &normalized_url);

    if webp_path.exists() {
        update_content_image(db_pool, content_id, &normalized_url).await?;
        return Ok(ProcessResult::Skipped(
            "WebP exists, updated DB".to_string(),
        ));
    }

    println!("   🔄 Converting: {} -> {}", image_url, normalized_url);
    logger
        .debug("images", &format!("Converting: {normalized_url}"))
        .await
        .ok();

    if let Some(parent) = webp_path.parent() {
        std::fs::create_dir_all(parent)
            .context(format!("Failed to create directory: {}", parent.display()))?;
    }

    convert_to_webp(&source_path, &webp_path)
        .await
        .context(format!("Failed to convert {}", source_path.display()))?;

    if !webp_path.exists() {
        anyhow::bail!("WebP file was not created: {}", webp_path.display());
    }

    update_content_image(db_pool, content_id, &normalized_url).await?;
    println!("   ✅ Converted: {} -> {}", image_url, normalized_url);

    Ok(ProcessResult::Optimized)
}

fn resolve_image_path(_web_public: &Path, image_url: &str) -> PathBuf {
    // UNIFIED FILES: All files live under STORAGE_PATH (default /app/files)
    // URL /files/images/blog/foo.png -> $STORAGE_PATH/images/blog/foo.png
    // URL /files/images/generated/2025/12/05/uuid.png ->
    // $STORAGE_PATH/images/generated/2025/12/05/uuid.png
    let storage_path = std::env::var("STORAGE_PATH").unwrap_or_else(|_| "/app/files".to_string());

    let relative_path = image_url
        .trim_start_matches('/')
        .trim_start_matches("files/");

    PathBuf::from(storage_path).join(relative_path)
}

async fn convert_to_webp(source: &Path, dest: &Path) -> Result<()> {
    // Try cwebp first (smaller, preferred), fall back to ffmpeg
    let output = Command::new("cwebp")
        .args([
            "-q",
            "80",
            "-m",
            "6",
            source.to_str().unwrap_or_default(),
            "-o",
            dest.to_str().unwrap_or_default(),
        ])
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => return Ok(()),
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            anyhow::bail!("cwebp failed: {}", stderr);
        },
        Err(_) => {
            // cwebp not available, try ffmpeg as fallback
            let ffmpeg_output = Command::new("ffmpeg")
                .args([
                    "-y",
                    "-i",
                    source.to_str().unwrap_or_default(),
                    "-c:v",
                    "libwebp",
                    "-quality",
                    "80",
                    "-compression_level",
                    "6",
                    dest.to_str().unwrap_or_default(),
                ])
                .output()
                .await
                .context("Neither cwebp nor ffmpeg available")?;

            if !ffmpeg_output.status.success() {
                let stderr = String::from_utf8_lossy(&ffmpeg_output.stderr);
                anyhow::bail!("ffmpeg failed: {}", stderr);
            }
        },
    }

    Ok(())
}

async fn update_content_image(
    db_pool: &DbPool,
    content_id: &str,
    new_image_url: &str,
) -> Result<()> {
    let pool = db_pool.pool_arc().context("Database must be PostgreSQL")?;
    let now = chrono::Utc::now();
    sqlx::query!(
        r#"UPDATE markdown_content SET image = $1, image_optimization_status = 'optimized',
         updated_at = $2 WHERE id = $3"#,
        new_image_url,
        now,
        content_id
    )
    .execute(pool.as_ref())
    .await
    .context("Failed to update content image")?;

    Ok(())
}
