use anyhow::{Context, Result};
use clap::Subcommand;
use std::env;
use std::sync::Arc;
use systemprompt_core_agent::services::a2a_server::Server;
use systemprompt_core_agent::services::agent_orchestration::AgentOrchestrator;
use systemprompt_core_logging::CliService;
use systemprompt_core_system::AppContext;
use systemprompt_models::repository::ServiceRepository;
use tokio::signal;

#[derive(Subcommand)]
pub enum AgentCommands {
    /// Enable and auto-start one or more A2A agents
    Enable {
        /// Agent name to enable
        agent_name: Option<String>,
        /// Enable all agents
        #[arg(long)]
        all: bool,
    },
    /// Disable and stop A2A agents (removes from services table)
    Disable {
        /// Agent name to disable
        agent_name: Option<String>,
        /// Disable all agents
        #[arg(long)]
        all: bool,
    },
    /// Restart a specific A2A agent (provide agent name)
    Restart {
        /// Agent name to restart
        agent_name: String,
    },
    /// Show detailed status of all registered A2A agents
    Status,
    /// List all registered agents with their current state
    List,
    /// Validate agent configuration and connectivity (provide agent name or use
    /// --all)
    Validate {
        /// Agent name to validate
        agent_name: Option<String>,
        /// Validate all agents
        #[arg(long)]
        all: bool,
    },
    /// Perform health checks on running agents (provide agent name or use
    /// --all)
    Health {
        /// Agent name to check
        agent_name: Option<String>,
        /// Check all agents
        #[arg(long)]
        all: bool,
    },
    /// Run orchestrator in daemon mode to monitor and maintain all agents
    Daemon {
        /// Health check interval in seconds
        #[arg(long, default_value = "30")]
        health_interval: u64,
    },
    /// Clean up orphaned processes and register them in the services table
    Cleanup,
    /// Delete agents from the system (provide agent name or use --all)
    Delete {
        /// Agent name to delete
        agent_name: Option<String>,
        /// Delete all agents
        #[arg(long)]
        all: bool,
    },
    /// Run agent A2A server (spawned by orchestrator)
    Run {
        /// Agent name (can also be set via AGENT_NAME env var)
        #[arg(long, env = "AGENT_NAME")]
        agent_name: String,
        /// Port to listen on
        #[arg(long, env = "AGENT_PORT")]
        port: u16,
    },
}

pub async fn execute(cmd: AgentCommands, ctx: Arc<AppContext>) -> Result<()> {
    env::set_var("SYSTEMPROMPT_NON_INTERACTIVE", "1");

    let orchestrator = AgentOrchestrator::new(ctx.clone())
        .await
        .context("Failed to initialize agent orchestrator")?;

    match cmd {
        AgentCommands::Enable { agent_name, all } => {
            if all {
                let service_ids = orchestrator.start_all().await?;
                CliService::success(&format!("Enabled {} agents", service_ids.len()));
            } else if let Some(name) = agent_name {
                let service_id = orchestrator.enable_agent(&name).await?;
                CliService::success(&format!("Agent enabled with service ID: {}", service_id));
            } else {
                return Err(anyhow::anyhow!("Please specify agent name or use --all"));
            }
        },
        AgentCommands::Disable { agent_name, all } => {
            if all {
                orchestrator.disable_all().await?;
                CliService::success("All agents disabled");
            } else if let Some(name) = agent_name {
                orchestrator.disable_agent(&name).await?;
                CliService::success(&format!("Agent {} disabled", name));
            } else {
                return Err(anyhow::anyhow!("Please specify agent name or use --all"));
            }
        },
        AgentCommands::Restart { agent_name } => {
            let service_id = orchestrator.restart_agent(&agent_name).await?;
            CliService::success(&format!(
                "Agent {} restarted with service ID: {}",
                agent_name, service_id
            ));
        },
        AgentCommands::Status => {
            orchestrator.show_detailed_status().await?;
        },
        AgentCommands::List => {
            orchestrator.show_detailed_status().await?;
        },
        AgentCommands::Validate { agent_name, all } => {
            if all {
                let all_agents = orchestrator.list_all().await?;
                CliService::info(&format!("Validating {} agents...", all_agents.len()));

                for (agent_name, _) in all_agents {
                    let report = orchestrator.validate_agent(&agent_name).await?;
                    if !report.valid {
                        CliService::error(&format!("{}: {}", agent_name, report.issues.join(", ")));
                    }
                }
            } else if let Some(name) = agent_name {
                orchestrator.validate_agent(&name).await?;
            } else {
                return Err(anyhow::anyhow!("Please specify agent name or use --all"));
            }
        },
        AgentCommands::Health { agent_name, all } => {
            if all {
                let reports = orchestrator.health_check_all().await?;
                CliService::success(&format!(
                    "Health check completed for {} agents",
                    reports.len()
                ));
            } else if let Some(name) = agent_name {
                let result = orchestrator.health_check(&name).await?;

                if result.healthy {
                    CliService::success(&format!(
                        "{}: {} ({}ms)",
                        name, result.message, result.response_time_ms
                    ));
                } else {
                    CliService::error(&format!(
                        "{}: {} ({}ms)",
                        name, result.message, result.response_time_ms
                    ));
                }
            } else {
                return Err(anyhow::anyhow!("Please specify agent name or use --all"));
            }
        },
        AgentCommands::Daemon { health_interval: _ } => {
            let mut orchestrator = orchestrator;
            orchestrator.run_daemon().await?;
        },
        AgentCommands::Cleanup => {
            orchestrator.cleanup_orphaned_processes().await?;
        },
        AgentCommands::Delete { agent_name, all } => {
            if all {
                let deleted_count = orchestrator.delete_all_agents().await?;
                CliService::success(&format!("Deleted {} agents", deleted_count));
            } else if let Some(name) = agent_name {
                orchestrator.delete_agent(&name).await?;
                CliService::success(&format!("Agent {} deleted", name));
            } else {
                return Err(anyhow::anyhow!("Please specify agent name or use --all"));
            }
        },
        AgentCommands::Run { agent_name, port } => {
            let name = agent_name;
            let logger = ctx.log.clone();
            let db_pool = ctx.db_pool().clone();

            let pid = std::process::id() as i32;
            let service_repo = ServiceRepository::new(db_pool.clone());
            service_repo
                .update_service_pid(&name, pid)
                .await
                .context("Failed to update service PID")?;

            service_repo
                .update_service_status(&name, "running")
                .await
                .context("Failed to update service status")?;

            let server =
                match Server::new(db_pool.clone(), ctx.clone(), Some(name.clone()), port).await {
                    Ok(s) => s,
                    Err(e) => {
                        logger
                            .error(
                                "agent_server",
                                &format!("Failed to create A2A server: {}", e),
                            )
                            .await
                            .ok();
                        return Err(e.context("Failed to create A2A server"));
                    },
                };

            let shutdown_service_repo = service_repo.clone();
            let shutdown_agent_name = name.clone();
            let shutdown = async move {
                signal::ctrl_c().await.ok();
                let _ = shutdown_service_repo
                    .update_service_status(&shutdown_agent_name, "stopped")
                    .await;
                let _ = shutdown_service_repo
                    .clear_service_pid(&shutdown_agent_name)
                    .await;
            };

            server
                .run_with_shutdown(shutdown)
                .await
                .context("Server failed during execution")?;

            logger
                .info("agent_server", "Server shutdown completed")
                .await
                .ok();
        },
    }

    Ok(())
}
