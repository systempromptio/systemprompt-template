use super::diff::SkillsDiffCalculator;
use super::export::export_skill_to_disk;
use super::models::{SkillsDiffResult, SyncDirection, SyncResult};
use super::SkillsSyncArgs;
use anyhow::{Context, Result};
use console::style;
use dialoguer::{Confirm, Select};
use std::path::PathBuf;
use std::sync::Arc;
use systemprompt_core_agent::repository::SkillRepository;
use systemprompt_core_agent::services::SkillIngestionService;
use systemprompt_core_database::{Database, DatabaseProvider};
use systemprompt_core_logging::CliService;
use systemprompt_identifiers::SourceId;

fn get_skills_path() -> Result<PathBuf> {
    let mut path = std::env::current_dir()?;
    path.push("crates/services/skills");
    Ok(path)
}

async fn create_db_provider(database_url: Option<&str>) -> Result<Arc<dyn DatabaseProvider>> {
    let url = match database_url {
        Some(url) => url.to_string(),
        None => std::env::var("DATABASE_URL").context("DATABASE_URL not set")?,
    };

    let database = Database::from_config("postgres", &url)
        .await
        .context("Failed to connect to database")?;

    Ok(Arc::new(database))
}

pub async fn execute(args: SkillsSyncArgs) -> Result<()> {
    CliService::section("Skills Sync");

    let db = create_db_provider(args.database_url.as_deref()).await?;
    let skills_path = get_skills_path()?;

    if !skills_path.exists() {
        CliService::warning(&format!(
            "Skills directory not found: {}",
            skills_path.display()
        ));
        return Ok(());
    }

    let calculator = SkillsDiffCalculator::new(db.clone());
    let diff = calculator
        .calculate_diff(&skills_path)
        .await
        .context("Failed to calculate skills diff")?;

    display_diff_summary(&diff);

    if !diff.has_changes() {
        CliService::success("Skills are in sync - no changes needed");
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
        SyncDirection::ToDisk => {
            sync_to_disk(&diff, db.clone(), &skills_path, args.delete_orphans).await?
        }
        SyncDirection::ToDatabase => {
            sync_to_db(&diff, db.clone(), &skills_path, args.delete_orphans).await?
        }
    };

    display_sync_result(&result);

    Ok(())
}

fn display_diff_summary(diff: &SkillsDiffResult) {
    println!();
    println!("{}", style("Skills Status").bold());
    println!("  {} unchanged", diff.unchanged);

    if !diff.added.is_empty() {
        println!(
            "  {} {} (on disk, not in DB)",
            style("+").green(),
            diff.added.len()
        );
        for item in &diff.added {
            println!(
                "    {} {} ({})",
                style("+").green(),
                item.skill_id,
                item.name.as_deref().unwrap_or("unnamed")
            );
        }
    }

    if !diff.removed.is_empty() {
        println!(
            "  {} {} (in DB, not on disk)",
            style("-").red(),
            diff.removed.len()
        );
        for item in &diff.removed {
            println!(
                "    {} {} ({})",
                style("-").red(),
                item.skill_id,
                item.name.as_deref().unwrap_or("unnamed")
            );
        }
    }

    if !diff.modified.is_empty() {
        println!(
            "  {} {} (modified)",
            style("~").yellow(),
            diff.modified.len()
        );
        for item in &diff.modified {
            println!(
                "    {} {} ({})",
                style("~").yellow(),
                item.skill_id,
                item.name.as_deref().unwrap_or("unnamed")
            );
        }
    }

    println!();
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
    diff: &SkillsDiffResult,
    db: Arc<dyn DatabaseProvider>,
    skills_path: &PathBuf,
    delete_orphans: bool,
) -> Result<SyncResult> {
    let skill_repo = SkillRepository::new(db);
    let mut result = SyncResult {
        direction: "to_disk".to_string(),
        ..Default::default()
    };

    for item in &diff.modified {
        let skill_id = systemprompt_identifiers::SkillId::new(&item.skill_id);
        match skill_repo.get_by_skill_id(&skill_id).await? {
            Some(skill) => {
                export_skill_to_disk(&skill, skills_path)?;
                result.items_synced += 1;
                println!("  {} Exported: {}", style("~").yellow(), item.skill_id);
            }
            None => {
                result
                    .errors
                    .push(format!("Skill not found in DB: {}", item.skill_id));
            }
        }
    }

    for item in &diff.removed {
        let skill_id = systemprompt_identifiers::SkillId::new(&item.skill_id);
        match skill_repo.get_by_skill_id(&skill_id).await? {
            Some(skill) => {
                export_skill_to_disk(&skill, skills_path)?;
                result.items_synced += 1;
                println!("  {} Created: {}", style("+").green(), item.skill_id);
            }
            None => {
                result
                    .errors
                    .push(format!("Skill not found in DB: {}", item.skill_id));
            }
        }
    }

    if delete_orphans {
        for item in &diff.added {
            let skill_dir_name = item.skill_id.replace('_', "-");
            let skill_dir = skills_path.join(&skill_dir_name);

            if skill_dir.exists() {
                std::fs::remove_dir_all(&skill_dir)?;
                result.items_deleted += 1;
                println!("  {} Deleted: {}", style("-").red(), item.skill_id);
            }
        }
    } else {
        for item in &diff.added {
            println!(
                "  {} Skipped delete: {} (use --delete-orphans)",
                style("!").cyan(),
                item.skill_id
            );
            result.items_skipped += 1;
        }
    }

    Ok(result)
}

async fn sync_to_db(
    diff: &SkillsDiffResult,
    db: Arc<dyn DatabaseProvider>,
    skills_path: &PathBuf,
    delete_orphans: bool,
) -> Result<SyncResult> {
    let ingestion_service = SkillIngestionService::new(db.clone());
    let mut result = SyncResult {
        direction: "to_database".to_string(),
        ..Default::default()
    };

    let source_id = SourceId::new("skills");
    let report = ingestion_service
        .ingest_directory(skills_path, source_id, true)
        .await?;

    result.items_synced += report.files_processed;

    for error in report.errors {
        result.errors.push(error);
    }

    println!(
        "  {} Ingested {} skills",
        style("✓").green(),
        report.files_processed
    );

    if delete_orphans {
        let skill_repo = SkillRepository::new(db);
        for item in &diff.removed {
            let skill_id = systemprompt_identifiers::SkillId::new(&item.skill_id);
            if let Some(skill) = skill_repo.get_by_skill_id(&skill_id).await? {
                result.items_deleted += 1;
                println!(
                    "  {} Would delete from DB: {} (delete not implemented)",
                    style("-").red(),
                    skill.skill_id.as_str()
                );
            }
        }
    } else {
        for item in &diff.removed {
            println!(
                "  {} Skipped delete: {} (use --delete-orphans)",
                style("!").cyan(),
                item.skill_id
            );
            result.items_skipped += 1;
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
