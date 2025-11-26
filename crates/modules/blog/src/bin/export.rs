use anyhow::{anyhow, Result};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use systemprompt_core_blog::{ContentSourceExport, ExportService};
use systemprompt_core_database::{Database, DatabaseProvider};
use systemprompt_core_logging::LogService;
use systemprompt_models::{Config, ContentConfig};

#[derive(Parser)]
#[command(name = "export")]
#[command(about = "Export database content to markdown files")]
#[command(
    long_about = "Exports content from the database to markdown files, syncing database as the \
source of truth back to the filesystem for backup and git tracking."
)]
struct Args {
    #[arg(short, long, default_value = "crates/services/content/config.yml")]
    config: PathBuf,

    #[arg(long)]
    source: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let args = Args::parse();
    let config = Config::from_env()?;
    let database = Database::from_config(&config.database_type, &config.database_url).await?;
    let db_arc = Arc::new(database);

    let logger = LogService::system(db_arc.clone());

    let content_config = ContentConfig::load_from_file(&args.config)?;

    if args.verbose {
        println!("\n📋 Content Sources Configuration:");
        println!("{}", serde_yaml::to_string(&content_config)?);
    }

    let db_provider: Arc<dyn DatabaseProvider> = db_arc.clone();
    let export_service = ExportService::new(db_provider);

    let sources_to_export: Vec<_> = if let Some(ref source_name) = args.source {
        content_config
            .content_sources
            .iter()
            .filter(|(name, config)| *name == source_name && config.enabled)
            .collect()
    } else {
        content_config
            .content_sources
            .iter()
            .filter(|(_, config)| config.enabled)
            .collect()
    };

    if sources_to_export.is_empty() {
        let msg = if args.source.is_some() {
            format!(
                "Source '{}' not found or disabled",
                args.source.as_ref().unwrap()
            )
        } else {
            "No enabled sources found in configuration".to_string()
        };
        return Err(anyhow!(msg));
    }

    println!("\n📤 Exporting {} source(s):", sources_to_export.len());
    for (name, config) in &sources_to_export {
        println!("  • {} ({})", name, config.description);
    }
    println!();

    let mut total_files_written = 0;
    let mut total_bytes_written = 0u64;
    let mut all_errors = Vec::new();

    for (source_name, source_config) in sources_to_export {
        println!("📂 Exporting source: {source_name}");

        let content_path = if source_config.path.starts_with('/') {
            PathBuf::from(&source_config.path)
        } else {
            std::env::current_dir()?.join(&source_config.path)
        };

        let export_source = ContentSourceExport {
            source_id: source_config.source_id.clone(),
            path: content_path.to_string_lossy().to_string(),
        };

        match export_service.export_source(&export_source).await {
            Ok(stats) => {
                total_files_written += stats.files_written;
                total_bytes_written += stats.bytes_written;

                println!("   ✅ {} files exported", stats.files_written);
                println!("   ✅ {} bytes written", stats.bytes_written);
            },
            Err(e) => {
                let err_msg = format!("Source '{source_name}' failed: {e}");
                println!("   ❌ {err_msg}");
                logger.error("content_export", &err_msg).await.ok();
                all_errors.push(err_msg);
            },
        }
    }

    println!("\n📊 Export Summary");
    println!("  Total files written: {total_files_written}");
    println!("  Total bytes written: {total_bytes_written}");

    if !all_errors.is_empty() {
        println!("\n⚠️  {} error(s) encountered:", all_errors.len());
        for error in &all_errors {
            println!("  - {error}");
        }
        println!("\n✨ Export completed with errors.");
    } else if total_files_written > 0 {
        println!("\n✨ All sources exported successfully!");
        println!("💾 Markdown files updated. Don't forget to commit changes to git.");
    } else {
        println!("\n⚠️  No files exported.");
    }

    Ok(())
}
