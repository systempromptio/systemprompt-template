use super::diff::ContentDiffCalculator;
use super::export::export_content_to_file;
use super::models::{ContentDiffResult, SyncDirection, SyncResult};
use super::ContentSyncArgs;
use anyhow::{Context, Result};
use console::style;
use dialoguer::{Confirm, Select};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use systemprompt_core_blog::repository::ContentRepository;
use systemprompt_core_blog::services::IngestionService;
use systemprompt_core_database::{Database, DbPool};
use systemprompt_core_logging::CliService;

#[derive(Debug, Clone, Deserialize)]
struct ContentConfig {
    content_sources: HashMap<String, ContentSource>,
}

#[derive(Debug, Clone, Deserialize)]
struct ContentSource {
    path: String,
    source_id: String,
    category_id: String,
    enabled: bool,
    #[serde(default)]
    allowed_content_types: Vec<String>,
}

struct DiffEntry {
    name: String,
    source: ContentSource,
    diff: ContentDiffResult,
}

fn get_content_config_path() -> Result<std::path::PathBuf> {
    let mut path = std::env::current_dir()?;
    path.push("crates/services/content/config.yml");
    if !path.exists() {
        anyhow::bail!("Content config not found at: {}", path.display());
    }
    Ok(path)
}

fn load_content_config() -> Result<ContentConfig> {
    let config_path = get_content_config_path()?;
    let content = std::fs::read_to_string(&config_path)
        .context(format!("Failed to read config: {}", config_path.display()))?;
    let config: ContentConfig =
        serde_yaml::from_str(&content).context("Failed to parse content config")?;
    Ok(config)
}

async fn create_db_pool(database_url: Option<&str>) -> Result<DbPool> {
    let url = match database_url {
        Some(url) => url.to_string(),
        None => std::env::var("DATABASE_URL").context("DATABASE_URL not set")?,
    };

    let database = Database::from_config("postgres", &url)
        .await
        .context("Failed to connect to database")?;

    Ok(std::sync::Arc::new(database))
}

pub async fn execute(args: ContentSyncArgs) -> Result<()> {
    CliService::section("Content Sync");

    let db = create_db_pool(args.database_url.as_deref()).await?;

    let config = load_content_config()?;

    let sources: Vec<(String, ContentSource)> = config
        .content_sources
        .into_iter()
        .filter(|(_, source)| source.enabled)
        .filter(|(name, _)| {
            if let Some(ref filter) = args.source {
                name.as_str() == filter.as_str()
            } else {
                true
            }
        })
        .filter(|(_, source)| !source.allowed_content_types.contains(&"skill".to_string()))
        .collect();

    if sources.is_empty() {
        if let Some(ref filter) = args.source {
            CliService::warning(&format!("No content source found matching: {}", filter));
        } else {
            CliService::warning("No enabled content sources found");
        }
        return Ok(());
    }

    let calculator = ContentDiffCalculator::new(db.clone());
    let mut all_diffs: Vec<DiffEntry> = Vec::new();

    for (name, source) in sources {
        CliService::info(&format!("Scanning source: {}", name));

        let base_path = std::env::current_dir()?;
        let source_path = base_path.join(&source.path);

        let diff = calculator
            .calculate_diff(&source.source_id, &source_path, &source.allowed_content_types)
            .await
            .context(format!("Failed to calculate diff for source: {}", name))?;

        all_diffs.push(DiffEntry {
            name,
            source,
            diff,
        });
    }

    display_diff_summary(&all_diffs);

    let has_changes = all_diffs.iter().any(|e| e.diff.has_changes());

    if !has_changes {
        CliService::success("Content is in sync - no changes needed");
        return Ok(());
    }

    let direction = if args.force_to_disk {
        SyncDirection::ToDisk
    } else if args.force_to_db {
        SyncDirection::ToDatabase
    } else {
        match prompt_sync_direction()? {
            Some(dir) => dir,
            None => {
                CliService::info("Sync cancelled");
                return Ok(());
            }
        }
    };

    if args.dry_run {
        CliService::info("Dry run - no changes made");
        return Ok(());
    }

    if !args.force_to_disk && !args.force_to_db {
        let confirmed = Confirm::new()
            .with_prompt("Proceed with sync?")
            .default(false)
            .interact()?;

        if !confirmed {
            CliService::info("Sync cancelled");
            return Ok(());
        }
    }

    let result = match direction {
        SyncDirection::ToDisk => sync_to_disk(&all_diffs, db.clone(), args.delete_orphans).await?,
        SyncDirection::ToDatabase => sync_to_db(&all_diffs, db.clone(), args.delete_orphans).await?,
    };

    display_sync_result(&result);

    Ok(())
}

fn display_diff_summary(diffs: &[DiffEntry]) {
    println!();

    for entry in diffs {
        println!("{}", style(format!("Source: {}", entry.name)).bold());
        println!("  {} unchanged", entry.diff.unchanged);

        if !entry.diff.added.is_empty() {
            println!(
                "  {} {} (on disk, not in DB)",
                style("+").green(),
                entry.diff.added.len()
            );
            for item in &entry.diff.added {
                println!("    {} {}", style("+").green(), item.slug);
            }
        }

        if !entry.diff.removed.is_empty() {
            println!(
                "  {} {} (in DB, not on disk)",
                style("-").red(),
                entry.diff.removed.len()
            );
            for item in &entry.diff.removed {
                println!("    {} {}", style("-").red(), item.slug);
            }
        }

        if !entry.diff.modified.is_empty() {
            println!(
                "  {} {} (modified)",
                style("~").yellow(),
                entry.diff.modified.len()
            );
            for item in &entry.diff.modified {
                println!("    {} {}", style("~").yellow(), item.slug);
            }
        }

        println!();
    }
}

fn prompt_sync_direction() -> Result<Option<SyncDirection>> {
    let options = vec![
        "Sync to disk (DB -> Disk)",
        "Sync to database (Disk -> DB)",
        "Cancel",
    ];

    let selection = Select::new()
        .with_prompt("Choose sync direction")
        .items(&options)
        .default(0)
        .interact()?;

    match selection {
        0 => Ok(Some(SyncDirection::ToDisk)),
        1 => Ok(Some(SyncDirection::ToDatabase)),
        _ => Ok(None),
    }
}

async fn sync_to_disk(
    diffs: &[DiffEntry],
    db: DbPool,
    delete_orphans: bool,
) -> Result<SyncResult> {
    let content_repo = ContentRepository::new(db);
    let mut result = SyncResult {
        direction: "to_disk".to_string(),
        ..Default::default()
    };

    for entry in diffs {
        let base_path = std::env::current_dir()?;
        let source_path = base_path.join(&entry.source.path);

        for item in &entry.diff.modified {
            match content_repo
                .get_by_source_and_slug(&entry.source.source_id, &item.slug)
                .await?
            {
                Some(content) => {
                    export_content_to_file(&content, &source_path, &entry.name)?;
                    result.items_synced += 1;
                    println!("  {} Exported: {}", style("~").yellow(), item.slug);
                }
                None => {
                    result
                        .errors
                        .push(format!("Content not found in DB: {}", item.slug));
                }
            }
        }

        for item in &entry.diff.removed {
            match content_repo
                .get_by_source_and_slug(&entry.source.source_id, &item.slug)
                .await?
            {
                Some(content) => {
                    export_content_to_file(&content, &source_path, &entry.name)?;
                    result.items_synced += 1;
                    println!("  {} Created: {}", style("+").green(), item.slug);
                }
                None => {
                    result
                        .errors
                        .push(format!("Content not found in DB: {}", item.slug));
                }
            }
        }

        if delete_orphans {
            for item in &entry.diff.added {
                let file_path = if entry.name == "blog" {
                    source_path.join(&item.slug).join("index.md")
                } else {
                    source_path.join(format!("{}.md", item.slug))
                };

                if file_path.exists() {
                    if entry.name == "blog" {
                        std::fs::remove_dir_all(source_path.join(&item.slug))?;
                    } else {
                        std::fs::remove_file(&file_path)?;
                    }
                    result.items_deleted += 1;
                    println!("  {} Deleted: {}", style("-").red(), item.slug);
                }
            }
        } else {
            for item in &entry.diff.added {
                println!(
                    "  {} Skipped delete: {} (use --delete-orphans)",
                    style("!").cyan(),
                    item.slug
                );
                result.items_skipped += 1;
            }
        }
    }

    Ok(result)
}

async fn sync_to_db(
    diffs: &[DiffEntry],
    db: DbPool,
    delete_orphans: bool,
) -> Result<SyncResult> {
    let ingestion_service = IngestionService::new(db.clone());
    let content_repo = ContentRepository::new(db);
    let mut result = SyncResult {
        direction: "to_database".to_string(),
        ..Default::default()
    };

    for entry in diffs {
        let base_path = std::env::current_dir()?;
        let source_path = base_path.join(&entry.source.path);

        let allowed_types: Vec<&str> = entry
            .source
            .allowed_content_types
            .iter()
            .map(|s| s.as_str())
            .collect();

        let report = ingestion_service
            .ingest_directory_with_types(
                Path::new(&source_path),
                Some(entry.source.source_id.clone()),
                Some(entry.source.category_id.clone()),
                Some("sync command".to_string()),
                Some("cli".to_string()),
                &allowed_types,
            )
            .await?;

        result.items_synced += report.files_processed;

        for error in report.errors {
            result.errors.push(error);
        }

        println!(
            "  {} Ingested {} files from {}",
            style("✓").green(),
            report.files_processed,
            entry.name
        );

        if delete_orphans {
            for item in &entry.diff.removed {
                content_repo
                    .delete(&item.slug)
                    .await
                    .context(format!("Failed to delete: {}", item.slug))?;
                result.items_deleted += 1;
                println!("  {} Deleted from DB: {}", style("-").red(), item.slug);
            }
        } else {
            for item in &entry.diff.removed {
                println!(
                    "  {} Skipped delete: {} (use --delete-orphans)",
                    style("!").cyan(),
                    item.slug
                );
                result.items_skipped += 1;
            }
        }
    }

    Ok(result)
}

fn display_sync_result(result: &SyncResult) {
    println!();
    CliService::section("Sync Complete");
    println!("  Direction: {}", result.direction);
    println!("  Synced: {}", result.items_synced);
    println!("  Deleted: {}", result.items_deleted);
    println!("  Skipped: {}", result.items_skipped);

    if !result.errors.is_empty() {
        println!();
        CliService::warning(&format!("Errors ({})", result.errors.len()));
        for error in &result.errors {
            println!("    {}", error);
        }
    }
}
