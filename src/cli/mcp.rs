use anyhow::{Context, Result};
use clap::Subcommand;
use std::env;
use std::sync::Arc;
use systemprompt_core_logging::CliService;
use systemprompt_core_mcp::services::McpManager;
use systemprompt_core_system::validate_system;
use systemprompt_core_system::AppContext;

#[derive(Subcommand)]
pub enum McpCommands {
    /// List all MCP services and their status
    List,
    /// Start MCP services (all or specific)
    Start {
        /// Specific service name to start (if not provided, starts all enabled services)
        service: Option<String>,
    },
    /// Stop MCP services (all or specific)
    Stop {
        /// Specific service name to stop (if not provided, stops all running services)
        service: Option<String>,
    },
    /// Build MCP services (all or specific)
    Build {
        /// Specific service name to build (if not provided, builds all services)
        service: Option<String>,
    },
    /// Restart MCP services (all or specific)
    Restart {
        /// Specific service name to restart
        service: Option<String>,
    },
    /// Show status of MCP services
    Status,
    /// Synchronize database state with actual running processes
    Sync,
    /// Validate MCP connection and list tools for a specific service
    Validate {
        /// Service name to validate
        service: String,
    },
    /// List enabled MCP package names for build scripts
    ListPackages,
}

pub async fn execute(cmd: McpCommands) -> Result<()> {
    env::set_var("SYSTEMPROMPT_NON_INTERACTIVE", "1");

    let ctx = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );

    match &cmd {
        McpCommands::List | McpCommands::Status | McpCommands::ListPackages => {
            // These commands only need to read existing state
        },
        _ => {
            CliService::info("Running system validation...");
            validate_system(&ctx)
                .await
                .context("System validation failed")?;
            CliService::success("System validation completed");
        },
    }

    let manager = McpManager::new(ctx)
        .await
        .context("Failed to initialize MCP manager")?;

    match cmd {
        McpCommands::List => {
            manager.list_services().await?;
        },
        McpCommands::Start { service } => {
            manager.start_services(service).await?;
        },
        McpCommands::Stop { service } => {
            manager.stop_services(service).await?;
        },
        McpCommands::Build { service } => {
            manager.build_services(service).await?;
        },
        McpCommands::Restart { service } => {
            manager.restart_services(service).await?;
        },
        McpCommands::Status => {
            manager.show_status().await?;
        },
        McpCommands::Sync => {
            manager.sync_database_state().await?;
        },
        McpCommands::Validate { service } => {
            manager.validate_service(&service).await?;
        },
        McpCommands::ListPackages => {
            manager.list_packages().await?;
        },
    }

    Ok(())
}
