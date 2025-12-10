use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::path::Path;
use systemprompt_core_database::DbPool;
use systemprompt_core_files::{File, FileMetadata, FileRepository};
use systemprompt_core_logging::{LogLevel, LogService};
use walkdir::WalkDir;

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "svg", "ico"];

pub async fn ingest_files(db_pool: DbPool, logger: LogService) -> Result<()> {
    let start_time = std::time::Instant::now();

    logger
        .info("scheduler", "Job started | job=file_ingestion")
        .await
        .ok();

    let storage_path = std::env::var("STORAGE_PATH").unwrap_or_else(|_| "/app/files".to_string());

    let images_dir = Path::new(&storage_path);

    if !images_dir.exists() {
        logger
            .warn(
                "file_ingestion",
                &format!("Images directory not found: {}", images_dir.display()),
            )
            .await
            .ok();
        return Ok(());
    }

    let file_repo = FileRepository::new(db_pool);

    let mut files_found = 0;
    let mut files_inserted = 0;
    let mut files_skipped = 0;
    let mut errors = 0;

    for entry in WalkDir::new(images_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        if !IMAGE_EXTENSIONS.contains(&extension.as_str()) {
            continue;
        }

        files_found += 1;

        let file_path = path.to_string_lossy().to_string();

        let public_url = path
            .strip_prefix(images_dir)
            .map(|p| format!("/images/{}", p.to_string_lossy()))
            .unwrap_or_else(|_| {
                format!(
                    "/images/{}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                )
            });

        match file_repo.get_by_path(&file_path).await {
            Ok(Some(_)) => {
                files_skipped += 1;
                continue;
            },
            Ok(None) => {},
            Err(e) => {
                logger
                    .error(
                        "file_ingestion",
                        &format!("Error checking file existence: {file_path} - {e}"),
                    )
                    .await
                    .ok();
                errors += 1;
                continue;
            },
        }

        let mime_type = mime_from_extension(&extension);
        let file_size = std::fs::metadata(path).map(|m| m.len() as i64).ok();
        let ai_content = path.to_string_lossy().contains("/generated/");

        let now = Utc::now();
        let metadata =
            serde_json::to_value(FileMetadata::default()).unwrap_or_else(|_| serde_json::json!({}));

        let file = File {
            id: uuid::Uuid::new_v4(),
            file_path,
            public_url: public_url.clone(),
            mime_type,
            file_size_bytes: file_size,
            ai_content,
            metadata,
            user_id: None,
            session_id: None,
            trace_id: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        match file_repo.insert_file(&file).await {
            Ok(_) => {
                files_inserted += 1;
            },
            Err(e) => {
                logger
                    .error(
                        "scheduler",
                        &format!("File ingestion failed | file={public_url}, error={e}"),
                    )
                    .await
                    .ok();
                errors += 1;
            },
        }
    }

    logger
        .log(
            LogLevel::Info,
            "scheduler",
            &format!(
                "Job completed | job=file_ingestion, files_inserted={}, total={}",
                files_inserted, files_found
            ),
            Some(json!({
                "job_name": "file_ingestion",
                "files_found": files_found,
                "files_inserted": files_inserted,
                "files_skipped": files_skipped,
                "errors": errors,
                "duration_ms": start_time.elapsed().as_millis(),
            })),
        )
        .await
        .ok();

    Ok(())
}

fn mime_from_extension(ext: &str) -> String {
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    }
    .to_string()
}
