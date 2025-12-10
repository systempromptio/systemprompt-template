use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use systemprompt_core_agent::SkillIngestionService;
use systemprompt_core_blog::GenericIngestionService;
use systemprompt_core_database::{DatabaseProvider, DbPool};
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::AppContext;
use systemprompt_models::ContentConfig;

use super::file_ingestion::ingest_files;
use super::skill_validation::validate_agent_skill_references;

pub async fn ingest_content(
    db_pool: DbPool,
    logger: LogService,
    _app_context: Arc<AppContext>,
) -> Result<()> {
    let start_time = std::time::Instant::now();

    logger
        .info("scheduler", "Job started | job=content_ingestion")
        .await
        .ok();

    let config_path = std::env::var("CONTENT_CONFIG_PATH")
        .unwrap_or_else(|_| "crates/services/content/config.yml".to_string());

    let config = ContentConfig::load_from_file(&config_path)
        .with_context(|| format!("Failed to load config: {config_path}"))?;

    let db_provider: Arc<dyn DatabaseProvider> = db_pool.clone();
    let generic_ingestion_service = GenericIngestionService::new(db_pool.clone());
    let skill_ingestion_service = SkillIngestionService::new(db_provider);

    let sources_to_ingest: Vec<_> = config
        .content_sources
        .iter()
        .filter(|(_, config)| config.enabled)
        .collect();

    if sources_to_ingest.is_empty() {
        logger
            .warn("scheduler", "No enabled content sources found")
            .await
            .ok();
        return Ok(());
    }

    logger
        .debug(
            "scheduler",
            &format!("Processing {} content source(s)", sources_to_ingest.len()),
        )
        .await
        .ok();

    let mut total_processed = 0;
    let mut total_errors = 0;

    for (source_name, source_config) in sources_to_ingest {
        logger
            .debug(
                "scheduler",
                &format!("Ingesting source | source={source_name}"),
            )
            .await
            .ok();

        let content_path = if source_config.path.starts_with('/') {
            PathBuf::from(&source_config.path)
        } else {
            std::env::current_dir()?.join(&source_config.path)
        };

        if !content_path.exists() {
            let err = format!("Source path not found: {}", content_path.display());
            logger.warn("scheduler", &err).await.ok();
            total_errors += 1;
            continue;
        }

        let override_existing = source_config
            .indexing
            .map(|i| i.override_existing)
            .unwrap_or(false);

        let report = if is_skill_source(source_name) {
            skill_ingestion_service
                .ingest_directory(
                    &content_path,
                    systemprompt_identifiers::SourceId::new(&source_config.source_id),
                    override_existing,
                )
                .await
        } else {
            if source_config.allowed_content_types.is_empty() {
                let err = format!(
                    "Content source '{}' has no allowed_content_types configured",
                    source_name
                );
                logger.error("scheduler", &err).await.ok();
                total_errors += 1;
                continue;
            }

            let allowed_types: Vec<&str> = source_config
                .allowed_content_types
                .iter()
                .map(|s| s.as_str())
                .collect();

            generic_ingestion_service
                .ingest_directory(
                    &content_path,
                    source_config.source_id.clone(),
                    source_config.category_id.clone(),
                    &allowed_types,
                    override_existing,
                )
                .await
        };

        match report {
            Ok(report) => {
                total_processed += report.files_processed;
                total_errors += report.errors.len();

                if !report.errors.is_empty() {
                    for error in &report.errors {
                        logger.warn("scheduler", error).await.ok();
                    }
                }

                logger
                    .log(
                        LogLevel::Debug,
                        "scheduler",
                        &format!(
                            "Source ingested | source={}, files_found={}, files_processed={}, \
                             errors={}",
                            source_name,
                            report.files_found,
                            report.files_processed,
                            report.errors.len()
                        ),
                        Some(serde_json::json!({
                            "source": source_name,
                            "files_found": report.files_found,
                            "files_processed": report.files_processed,
                            "error_count": report.errors.len(),
                        })),
                    )
                    .await
                    .ok();
            },
            Err(e) => {
                let err_msg = format!(
                    "Source ingestion failed | source={}, error={}",
                    source_name, e
                );
                logger.error("scheduler", &err_msg).await.ok();
                total_errors += 1;
            },
        }
    }

    logger
        .log(
            LogLevel::Info,
            "scheduler",
            &format!(
                "Job completed | job=content_ingestion, files_processed={}, errors={}",
                total_processed, total_errors
            ),
            Some(serde_json::json!({
                "job_name": "content_ingestion",
                "files_processed": total_processed,
                "errors": total_errors,
                "sources_count": total_processed + total_errors,
                "duration_ms": start_time.elapsed().as_millis(),
            })),
        )
        .await
        .ok();

    // Ingest files from the images directory
    if let Err(e) = ingest_files(db_pool.clone(), logger.clone()).await {
        logger
            .error("scheduler", &format!("File ingestion failed: {e}"))
            .await
            .ok();
    }

    match validate_agent_skill_references(&db_pool, &logger).await {
        Ok(()) => {
            logger
                .debug("scheduler", "Skill references validated | status=success")
                .await
                .ok();
        },
        Err(e) => {
            let err_msg = format!("Skill validation failed | error={e}");
            logger.error("scheduler", &err_msg).await.ok();
            println!("{}", err_msg);
            return Err(e);
        },
    }

    Ok(())
}

fn is_skill_source(source_name: &str) -> bool {
    source_name.contains("skill")
}
