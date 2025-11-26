use anyhow::Result;
use clap::Subcommand;
use std::sync::Arc;
use systemprompt_core_logging::{CliService, LogService};
use systemprompt_core_scheduler::services::jobs;
use systemprompt_core_system::repository::AnalyticsSessionRepository;
use systemprompt_core_system::AppContext;

#[derive(Subcommand)]
pub enum SchedulerCommands {
    /// Run a scheduled job manually
    Run {
        /// Job name to run
        job_name: String,
    },
    /// Clean up inactive sessions
    CleanupSessions {
        /// Hours of inactivity threshold (default: 1)
        #[arg(long, default_value = "1")]
        hours: i32,
    },
}

pub async fn execute(cmd: SchedulerCommands, ctx: Arc<AppContext>) -> Result<()> {
    match cmd {
        SchedulerCommands::Run { job_name } => run_job(&job_name, ctx).await,
        SchedulerCommands::CleanupSessions { hours } => cleanup_sessions(hours, ctx).await,
    }
}

async fn run_job(job_name: &str, ctx: Arc<AppContext>) -> Result<()> {
    println!("🤖 Running job: {job_name}");

    let db_pool = ctx.db_pool().clone();
    let logger = LogService::system(db_pool.clone());

    let result = match job_name {
        "evaluate_conversations" => jobs::evaluate_conversations(db_pool, logger, ctx).await,
        "regenerate_static_content" => jobs::regenerate_static_content(db_pool, logger, ctx).await,
        "ingest_content" => jobs::ingest_content(db_pool, logger, ctx).await,
        "cleanup_anonymous_users" => jobs::cleanup_anonymous_users(db_pool, logger, ctx).await,
        "cleanup_inactive_sessions" => jobs::cleanup_inactive_sessions(db_pool, logger, ctx).await,
        "database_cleanup" => jobs::database_cleanup(db_pool, logger, ctx).await,
        "rebuild_static_site" => jobs::rebuild_static_site(db_pool, logger, ctx).await,
        _ => {
            eprintln!("❌ Unknown job: {job_name}");
            anyhow::bail!("Unknown job: {job_name}")
        },
    };

    match result {
        Ok(_) => {
            println!("✅ Job completed successfully");
            Ok(())
        },
        Err(e) => {
            eprintln!("❌ Job failed: {}", e);
            Err(e)
        },
    }
}

async fn cleanup_sessions(hours: i32, ctx: Arc<AppContext>) -> Result<()> {
    CliService::section("Session Cleanup");

    CliService::info(&format!(
        "🧹 Cleaning up sessions inactive for >{} hour(s)...",
        hours
    ));

    let session_repo = AnalyticsSessionRepository::new(ctx.db_pool().clone());
    let closed_count = session_repo.cleanup_inactive_sessions(hours).await?;

    CliService::success(&format!("✅ Closed {} inactive session(s)", closed_count));

    Ok(())
}
