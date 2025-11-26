use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Args;
use colored::Colorize;
use std::time::Duration;
use systemprompt_core_logging::{
    models::{LogEntry, LogLevel},
    repository::LoggingRepository,
};
use systemprompt_core_system::models::AppContext;
use tokio::time;

#[derive(Args)]
pub struct LogsArgs {
    /// Filter by log level (ERROR, WARN, INFO, DEBUG, TRACE)
    #[arg(long)]
    level: Option<String>,

    /// Filter by module name
    #[arg(long)]
    module: Option<String>,

    /// Number of initial logs to show
    #[arg(long, default_value = "20")]
    tail: i64,

    /// Stream logs continuously (default: show logs once and exit)
    #[arg(long, short = 's', action = clap::ArgAction::SetTrue)]
    stream: bool,

    /// Refresh interval in milliseconds (only used with --stream)
    #[arg(long, default_value = "1000")]
    interval: u64,

    /// Clear terminal before each refresh
    #[arg(long)]
    clear: bool,

    /// Clear all logs from database
    #[arg(long, action = clap::ArgAction::SetTrue)]
    clear_all: bool,

    /// Cleanup logs older than specified days
    #[arg(long)]
    cleanup: bool,

    /// Delete logs older than N days (use with --cleanup)
    #[arg(long)]
    older_than: Option<i64>,

    /// Keep only last N days of logs (use with --cleanup)
    #[arg(long)]
    keep_last_days: Option<i64>,

    /// Run VACUUM after cleanup to reclaim disk space
    #[arg(long, action = clap::ArgAction::SetTrue)]
    vacuum: bool,

    /// Show what would be deleted without actually deleting (use with --cleanup)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    dry_run: bool,
}

async fn get_initial_logs(repo: &LoggingRepository, args: &LogsArgs) -> Result<Vec<LogEntry>> {
    // TODO: Implement filtering by level/module using get_logs_paginated
    // The get_logs_by_level and get_logs_by_module methods were removed in Log module refactoring
    if args.level.is_some() || args.module.is_some() {
        eprintln!(
            "Warning: Filtering by level/module not yet implemented in refactored Log module"
        );
    }
    repo.get_recent_logs(args.tail)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get initial logs: {}", e))
}

async fn get_new_logs(
    repo: &LoggingRepository,
    args: &LogsArgs,
    since: DateTime<Utc>,
) -> Result<Vec<LogEntry>> {
    let all_recent_logs = repo
        .get_recent_logs(100)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get recent logs: {}", e))?;

    let mut new_logs: Vec<LogEntry> = all_recent_logs
        .into_iter()
        .filter(|log| log.timestamp > since)
        .collect();

    if let Some(ref level_str) = args.level {
        let level: LogLevel = level_str
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid log level: {}", level_str))?;
        new_logs.retain(|log| log.level == level);
    }

    if let Some(ref module) = args.module {
        new_logs.retain(|log| log.module.contains(module));
    }

    new_logs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    Ok(new_logs)
}

fn print_log(log: &LogEntry) {
    let timestamp = log
        .timestamp
        .format("%H:%M:%S%.3f")
        .to_string()
        .bright_black();
    let module = format!("[{}]", log.module).bright_blue();

    let (_level_str, level_color) = match log.level {
        LogLevel::Error => ("ERROR", log.level.to_string().bright_red().bold()),
        LogLevel::Warn => ("WARN ", log.level.to_string().bright_yellow()),
        LogLevel::Info => ("INFO ", log.level.to_string().bright_green()),
        LogLevel::Debug => ("DEBUG", log.level.to_string().bright_cyan()),
        LogLevel::Trace => ("TRACE", log.level.to_string().bright_purple()),
    };

    let message = if log.level == LogLevel::Error {
        log.message.bright_red()
    } else {
        log.message.normal()
    };

    if let Some(ref metadata) = log.metadata {
        println!(
            "{} {} {} {} {}",
            timestamp,
            level_color,
            module,
            message,
            serde_json::to_string(metadata)
                .unwrap_or_default()
                .bright_black()
        );
    } else {
        println!("{} {} {} {}", timestamp, level_color, module, message);
    }
}

async fn execute_cleanup(repo: &LoggingRepository, args: &LogsArgs) -> Result<()> {
    let days = if let Some(days) = args.older_than {
        days
    } else if let Some(keep_days) = args.keep_last_days {
        keep_days
    } else {
        return Err(anyhow::anyhow!(
            "Please specify --older-than <DAYS> or --keep-last-days <DAYS>"
        ));
    };

    let cutoff = Utc::now() - chrono::Duration::days(days);

    println!("{}", "🗑️  Log Cleanup".bright_cyan().bold());
    println!("{}", "━".repeat(60).bright_blue());

    if let Some(ref level) = args.level {
        println!("   Level:    {}", level.bright_yellow());
    }
    if let Some(ref module) = args.module {
        println!("   Module:   {}", module.bright_yellow());
    }
    println!(
        "   Cutoff:   {} ({} days ago)",
        cutoff
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .bright_yellow(),
        days.to_string().bright_yellow()
    );

    if args.dry_run {
        println!(
            "{}",
            "\n⚠️  DRY RUN MODE - No logs will be deleted\n"
                .bright_yellow()
                .bold()
        );
    }

    println!();

    let deleted = if !args.dry_run {
        repo.cleanup_old_logs(cutoff).await?
    } else {
        repo.get_recent_logs(1000)
            .await?
            .iter()
            .filter(|log| log.timestamp < cutoff)
            .count() as u64
    };

    println!("{}", "Results:".bright_green().bold());
    println!(
        "   {} logs to be deleted",
        deleted.to_string().bright_red().bold()
    );

    if !args.dry_run {
        println!("{}", "\n✅ Cleanup complete!".bright_green().bold());

        if args.vacuum {
            println!("\n🔧 Running VACUUM to reclaim disk space...");
            // TODO: VACUUM method removed in Log module refactoring
            // repo.vacuum().await?;
            println!("⚠️  VACUUM not yet implemented in refactored Log module");
        }
    } else {
        println!(
            "{}",
            "\nℹ️  Run without --dry-run to actually delete logs".bright_yellow()
        );
    }

    Ok(())
}

pub async fn execute(args: LogsArgs) -> Result<()> {
    dotenvy::dotenv().ok();

    let ctx = AppContext::new().await?;
    let repo = LoggingRepository::new(ctx.db_pool().clone());

    if args.clear_all {
        let cleared = repo.clear_all_logs().await?;
        println!("✅ Cleared {} log entries", cleared);

        if args.vacuum {
            println!("🔧 Running VACUUM to reclaim disk space...");
            // TODO: VACUUM method removed in Log module refactoring
            // repo.vacuum().await?;
            println!("⚠️  VACUUM not yet implemented in refactored Log module");
        }

        return Ok(());
    }

    if args.cleanup {
        return execute_cleanup(&repo, &args).await;
    }

    let mut last_timestamp: Option<DateTime<Utc>> = None;

    println!("{}", "SystemPrompt Log Stream".bright_cyan().bold());
    println!("{}", "─".repeat(50).bright_blue());

    if let Some(ref level) = args.level {
        println!("Filtering by level: {}", level.bright_yellow());
    }
    if let Some(ref module) = args.module {
        println!("Filtering by module: {}", module.bright_yellow());
    }
    if args.stream {
        println!(
            "Streaming mode: enabled (refresh interval: {}ms)",
            args.interval.to_string().bright_green()
        );
    }
    println!();

    loop {
        if args.clear {
            print!("\x1B[2J\x1B[1;1H");
            println!("{}", "SystemPrompt Log Stream".bright_cyan().bold());
            println!("{}", "─".repeat(50).bright_blue());
        }

        let logs = if last_timestamp.is_none() {
            get_initial_logs(&repo, &args).await?
        } else {
            get_new_logs(&repo, &args, last_timestamp.unwrap()).await?
        };

        if !logs.is_empty() {
            for log in &logs {
                print_log(log);
            }
            last_timestamp = logs.iter().map(|log| log.timestamp).max();
        } else if last_timestamp.is_none() {
            println!("{}", "No logs found".bright_yellow());
        }

        if !args.stream {
            break;
        }

        time::sleep(Duration::from_millis(args.interval)).await;
    }

    Ok(())
}
