use anyhow::{Context, Result};
use clap::Subcommand;
use std::sync::Arc;
use systemprompt_core_logging::CliService;
use systemprompt_core_system::{validate_system, AppContext, ServiceCategory};

#[derive(Subcommand)]
pub enum ServeCommands {
    /// Start API server (automatically starts all enabled agents and MCP
    /// servers)
    #[command(long_about = "Start the API server in foreground or daemon mode.

This command automatically:
  - Starts all enabled agents
  - Starts all enabled MCP servers
  - Spawns background monitors for auto-restart
  - Starts the API server

Use --foreground to run in the terminal (recommended for development).
Use daemon mode (default) for production deployments.")]
    Api {
        #[arg(long, help = "Run in foreground mode")]
        foreground: bool,
    },
}

pub async fn execute(cmd: ServeCommands) -> Result<()> {
    match cmd {
        ServeCommands::Api { foreground } => serve_api(foreground).await,
    }
}

async fn serve_api(foreground: bool) -> Result<()> {
    CliService::info("🔍 Pre-flight check: verifying port 8080 is available...");
    if let Some(pid) = crate::cli::cleanup::check_port_available(8080) {
        CliService::error(&format!("❌ Port 8080 is already in use by PID {}", pid));
        CliService::info("💡 To restart cleanly, run one of:");
        CliService::info("   - just api-rebuild    (rebuild and restart)");
        CliService::info("   - just api-nuke       (nuclear option - kill everything)");
        return Err(anyhow::anyhow!(
            "Port 8080 is occupied by PID {}. Cannot start server.",
            pid
        ));
    }
    CliService::success("✅ Port 8080 is available");

    let api_registrations: Vec<_> =
        inventory::iter::<systemprompt_core_system::models::ModuleApiRegistration>().collect();
    CliService::info(&format!(
        "Loading {} route modules for inventory registration:",
        api_registrations.len()
    ));
    for registration in &api_registrations {
        let category_name = match registration.category {
            ServiceCategory::Core => "Core",
            ServiceCategory::Agent => "Agent",
            ServiceCategory::Mcp => "Mcp",
            ServiceCategory::Meta => "Meta",
        };
        CliService::info(&format!(
            "   - {} ({}) - {} proxy routes",
            registration.module_name, category_name, registration.module_name
        ));
    }

    let ctx = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );

    CliService::info("Running system validation...");
    validate_system(&ctx)
        .await
        .context("System validation failed")?;

    if foreground {
        systemprompt_core_api::services::server::run_server(Arc::unwrap_or_clone(ctx)).await?;
    } else {
        CliService::warning("Daemon mode not currently supported, running in foreground");
        systemprompt_core_api::services::server::run_server(Arc::unwrap_or_clone(ctx)).await?;
    }

    Ok(())
}
