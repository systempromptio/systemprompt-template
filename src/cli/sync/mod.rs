use clap::{Args, Subcommand};

pub mod content;
pub mod diff;
pub mod export;
pub mod models;
pub mod skills;

#[derive(Subcommand)]
pub enum SyncCommands {
    /// Sync content (blog, legal) between disk and database
    Content(ContentSyncArgs),
    /// Sync skills between disk and database
    Skills(SkillsSyncArgs),
}

#[derive(Args)]
pub struct ContentSyncArgs {
    /// Force sync from database to disk (non-interactive)
    #[arg(long, conflicts_with = "force_to_db")]
    pub force_to_disk: bool,

    /// Force sync from disk to database (non-interactive)
    #[arg(long, conflicts_with = "force_to_disk")]
    pub force_to_db: bool,

    /// Override DATABASE_URL for target database
    #[arg(long)]
    pub database_url: Option<String>,

    /// Specific source to sync (e.g., "blog", "legal")
    #[arg(long)]
    pub source: Option<String>,

    /// Dry run - show what would change without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Delete items that exist only on target (orphans)
    #[arg(long)]
    pub delete_orphans: bool,
}

#[derive(Args)]
pub struct SkillsSyncArgs {
    /// Force sync from database to disk (non-interactive)
    #[arg(long, conflicts_with = "force_to_db")]
    pub force_to_disk: bool,

    /// Force sync from disk to database (non-interactive)
    #[arg(long, conflicts_with = "force_to_disk")]
    pub force_to_db: bool,

    /// Override DATABASE_URL for target database
    #[arg(long)]
    pub database_url: Option<String>,

    /// Specific skill to sync (by skill_id)
    #[arg(long)]
    pub skill: Option<String>,

    /// Dry run - show what would change without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Delete items that exist only on target (orphans)
    #[arg(long)]
    pub delete_orphans: bool,
}

pub async fn execute(cmd: SyncCommands) -> anyhow::Result<()> {
    match cmd {
        SyncCommands::Content(args) => content::execute(args).await,
        SyncCommands::Skills(args) => skills::execute(args).await,
    }
}
