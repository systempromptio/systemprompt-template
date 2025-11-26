use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::sync::Arc;
use systemprompt_core_system::{AppContext, Config};

mod cli;

#[derive(Parser)]
#[command(name = "systemprompt")]
#[command(
    about = "SystemPrompt OS - Unified CLI for agent orchestration, AI operations, and system management"
)]
#[command(version = "0.1.0")]
#[command(long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(long, short = 'v', global = true)]
    verbose: bool,

    /// Enable quiet mode (errors only)
    #[arg(long, short = 'q', global = true, conflicts_with = "verbose")]
    quiet: bool,

    /// Enable debug output
    #[arg(long, global = true)]
    debug: bool,

    /// Output in JSON format
    #[arg(long, global = true)]
    json: bool,

    /// Output in YAML format
    #[arg(long, global = true, conflicts_with = "json")]
    yaml: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Non-interactive mode
    #[arg(long, global = true)]
    non_interactive: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage system services
    ///
    /// Start the API server, which automatically manages all enabled agents and MCP servers.
    /// For manual control, use 'agents' and 'mcp' commands.
    #[command(subcommand)]
    Serve(cli::serve::ServeCommands),

    /// Agent orchestration and lifecycle management
    ///
    /// Manual control for agents. When API server is running, agents auto-start.
    /// Use these commands for manual operations or when API is not running.
    #[command(subcommand)]
    Agents(cli::agents::AgentCommands),

    /// MCP server management
    ///
    /// Manual control for MCP servers. When API server is running, MCP servers auto-start.
    /// Use these commands for manual operations or when API is not running.
    #[command(subcommand)]
    Mcp(cli::mcp::McpCommands),

    /// Database operations
    #[command(subcommand)]
    Db(cli::db::DbCommands),

    /// AI operations and inference
    #[command(subcommand)]
    Ai(cli::ai::AiCommands),

    /// Scheduler and background jobs
    #[command(subcommand)]
    Scheduler(cli::scheduler::SchedulerCommands),

    /// User analytics and reporting
    User(cli::user::UserArgs),

    /// Log streaming and inspection
    Logs(cli::logs::LogsArgs),

    /// Trace viewer
    Trace {
        trace_id: String,
        #[command(flatten)]
        options: cli::trace::TraceOptions,
    },

    /// Generate authentication tokens
    Login(cli::login::LoginArgs),

    /// System setup wizard
    Setup(cli::setup::SetupArgs),

    /// Sync skills to Claude Code
    Skills(cli::skills::SkillsArgs),

    /// Show status of all services (API, agents, MCP servers)
    Status,

    /// Restart services (API, agents, or MCP servers)
    Restart {
        #[command(subcommand)]
        target: Option<cli::restart::RestartTarget>,

        /// Restart all failed services
        #[arg(long)]
        failed: bool,
    },

    /// Clean up all running services (agents, MCP, API)
    #[command(name = "cleanup-services")]
    CleanupServices,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let cli_config = build_cli_config(&cli);
    cli::config::set_global_config(cli_config.clone());

    if cli.no_color || !cli_config.should_use_color() {
        console::set_colors_enabled(false);
    }

    Config::init().context("Failed to initialize configuration")?;

    match cli.command {
        Commands::Serve(cmd) => {
            cli::serve::execute(cmd).await?;
        },
        Commands::Agents(cmd) => {
            let ctx = Arc::new(
                AppContext::new()
                    .await
                    .context("Failed to initialize application context")?,
            );
            cli::agents::execute(cmd, ctx).await?;
        },
        Commands::Mcp(cmd) => {
            cli::mcp::execute(cmd).await?;
        },
        Commands::Db(cmd) => {
            cli::db::execute(cmd).await?;
        },
        Commands::Ai(cmd) => {
            cli::ai::execute(cmd).await?;
        },
        Commands::Scheduler(cmd) => {
            let ctx = Arc::new(
                AppContext::new()
                    .await
                    .context("Failed to initialize application context")?,
            );
            cli::scheduler::execute(cmd, ctx).await?;
        },
        Commands::User(args) => {
            cli::user::execute(args).await?;
        },
        Commands::Logs(args) => {
            cli::logs::execute(args).await?;
        },
        Commands::Trace { trace_id, options } => {
            cli::trace::execute(&trace_id, options).await?;
        },
        Commands::Login(args) => {
            cli::login::execute(args).await?;
        },
        Commands::Setup(args) => {
            cli::setup::execute(args).await?;
        },
        Commands::Skills(cmd) => {
            cli::skills::execute(cmd).await?;
        },
        Commands::Status => {
            cli::status::execute().await?;
        },
        Commands::Restart { target, failed } => {
            cli::restart::execute(target, failed).await?;
        },
        Commands::CleanupServices => {
            cli::cleanup::execute().await?;
        },
    }

    Ok(())
}

fn build_cli_config(cli: &Cli) -> cli::config::CliConfig {
    let mut config = cli::config::CliConfig::new();

    if cli.debug {
        config = config.with_verbosity(cli::config::VerbosityLevel::Debug);
    } else if cli.verbose {
        config = config.with_verbosity(cli::config::VerbosityLevel::Verbose);
    } else if cli.quiet {
        config = config.with_verbosity(cli::config::VerbosityLevel::Quiet);
    }

    if cli.json {
        config = config.with_output_format(cli::config::OutputFormat::Json);
    } else if cli.yaml {
        config = config.with_output_format(cli::config::OutputFormat::Yaml);
    }

    if cli.no_color {
        config = config.with_color_mode(cli::config::ColorMode::Never);
    }

    if cli.non_interactive {
        config = config.with_interactive(false);
    }

    config
}
