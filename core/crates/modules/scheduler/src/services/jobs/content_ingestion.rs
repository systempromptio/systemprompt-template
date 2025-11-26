use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use systemprompt_core_agent::SkillIngestionService;
use systemprompt_core_blog::GenericIngestionService;
use systemprompt_core_database::{DatabaseProvider, DbPool};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;
use systemprompt_models::ContentConfig;

use super::skill_validation::validate_agent_skill_references;

const DEFAULT_ALLOWED_CONTENT_TYPES: &[&str] =
    &["article", "post", "page", "blog", "documentation"];

pub async fn ingest_content(
    db_pool: DbPool,
    logger: LogService,
    _app_context: Arc<AppContext>,
) -> Result<()> {
    logger
        .info(
            "scheduler",
            "Starting scheduled content and skills ingestion",
        )
        .await
        .ok();

    let config_path = std::env::var("CONTENT_CONFIG_PATH")
        .unwrap_or_else(|_| "crates/services/content/config.yml".to_string());

    let config = ContentConfig::load_from_file(&config_path)
        .with_context(|| format!("Failed to load config: {}", config_path))?;

    let db_provider: Arc<dyn DatabaseProvider> = db_pool.clone();
    let generic_ingestion_service = GenericIngestionService::new(db_provider.clone());
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
        .info(
            "scheduler",
            &format!("Processing {} content source(s)", sources_to_ingest.len()),
        )
        .await
        .ok();

    let mut total_processed = 0;
    let mut total_errors = 0;

    for (source_name, source_config) in sources_to_ingest {
        let icon = if is_skill_source(source_name) {
            "🎯"
        } else {
            "📂"
        };
        println!("{} Ingesting source: {}", icon, source_name);
        logger
            .info("scheduler", &format!("Ingesting source: {}", source_name))
            .await
            .ok();

        let content_path = if source_config.path.starts_with('/') {
            PathBuf::from(&source_config.path)
        } else {
            std::env::current_dir()?.join(&source_config.path)
        };

        if !content_path.exists() {
            let err = format!("Source path not found: {}", content_path.display());
            println!("   ⚠️  {}", err);
            logger.warn("scheduler", &err).await.ok();
            total_errors += 1;
            continue;
        }

        let report = if is_skill_source(source_name) {
            skill_ingestion_service
                .ingest_directory(
                    &content_path,
                    systemprompt_identifiers::SourceId::new(&source_config.source_id),
                )
                .await
        } else {
            let allowed_types: Vec<&str> = if source_config.allowed_content_types.is_empty() {
                DEFAULT_ALLOWED_CONTENT_TYPES.to_vec()
            } else {
                source_config
                    .allowed_content_types
                    .iter()
                    .map(|s| s.as_str())
                    .collect()
            };

            generic_ingestion_service
                .ingest_directory(
                    &content_path,
                    source_config.source_id.clone(),
                    source_config.category_id.clone(),
                    &allowed_types,
                )
                .await
        };

        match report {
            Ok(report) => {
                total_processed += report.files_processed;
                total_errors += report.errors.len();

                let summary = format!(
                    "✅ {} files found\n   ✅ {} files processed",
                    report.files_found, report.files_processed
                );
                println!("   {}", summary);

                if !report.errors.is_empty() {
                    println!("   ⚠️  {} errors encountered:", report.errors.len());
                    for error in &report.errors {
                        println!("      - {}", error);
                        logger.warn("scheduler", error).await.ok();
                    }
                }

                logger
                    .info(
                        "scheduler",
                        &format!(
                            "Source '{}' complete: {} files found, {} files processed, {} errors",
                            source_name,
                            report.files_found,
                            report.files_processed,
                            report.errors.len()
                        ),
                    )
                    .await
                    .ok();
            },
            Err(e) => {
                let err_msg = format!("Source '{}' failed: {}", source_name, e);
                println!("   ❌ {}", err_msg);
                logger.error("scheduler", &err_msg).await.ok();
                total_errors += 1;
            },
        }
    }

    logger
        .info(
            "scheduler",
            &format!(
                "Content and skills ingestion complete: {} files processed, {} errors",
                total_processed, total_errors
            ),
        )
        .await
        .ok();

    logger
        .info("scheduler", "Validating agent skill references...")
        .await
        .ok();

    match validate_agent_skill_references(&db_pool, &logger).await {
        Ok(()) => {
            logger
                .info(
                    "scheduler",
                    "All agent skill references validated successfully",
                )
                .await
                .ok();
        },
        Err(e) => {
            let err_msg = format!("FATAL: Agent skill validation failed: {}", e);
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
