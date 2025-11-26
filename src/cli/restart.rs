use anyhow::{Context, Result};
use clap::Subcommand;
use std::sync::Arc;
use systemprompt_core_agent::services::agent_orchestration::AgentOrchestrator;
use systemprompt_core_agent::services::registry::AgentRegistry;
use systemprompt_core_logging::CliService;
use systemprompt_core_mcp::services::McpManager;
use systemprompt_core_system::AppContext;

async fn resolve_name(agent_identifier: &str) -> Result<String> {
    let registry = AgentRegistry::new().await?;
    let agent = registry.get_agent(agent_identifier).await?;
    Ok(agent.name)
}

#[derive(Subcommand)]
pub enum RestartTarget {
    /// Restart API server (stops and restarts all services)
    Api,

    /// Restart specific agent
    Agent {
        /// Agent ID or name to restart
        agent_id: String,
    },

    /// Restart specific MCP server
    Mcp {
        /// MCP server name to restart
        server_name: String,
    },
}

pub async fn execute(target: Option<RestartTarget>, failed: bool) -> Result<()> {
    let ctx = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );

    if failed {
        restart_failed_services(&ctx).await?;
        return Ok(());
    }

    match target {
        Some(RestartTarget::Api) => restart_api(&ctx).await?,
        Some(RestartTarget::Agent { agent_id }) => restart_agent(&ctx, &agent_id).await?,
        Some(RestartTarget::Mcp { server_name }) => restart_mcp(&ctx, &server_name).await?,
        None => {
            return Err(anyhow::anyhow!(
                "Must specify target (api, agent, mcp) or use --failed flag"
            ));
        },
    }

    Ok(())
}

async fn restart_api(_ctx: &Arc<AppContext>) -> Result<()> {
    CliService::section("Restarting API Server");

    CliService::warning("API server restart via CLI is not currently supported");
    CliService::info("To restart the API server:");
    CliService::info("  1. Stop the current server (Ctrl+C if running in foreground)");
    CliService::info("  2. Run: just api");

    Ok(())
}

async fn restart_agent(ctx: &Arc<AppContext>, agent_id: &str) -> Result<()> {
    CliService::section(&format!("Restarting Agent: {}", agent_id));

    let orchestrator = AgentOrchestrator::new(ctx.clone())
        .await
        .context("Failed to initialize agent orchestrator")?;

    let name = resolve_name(agent_id).await?;
    let service_id = orchestrator.restart_agent(&name).await?;

    CliService::success(&format!(
        "✅ Agent {} restarted successfully (service ID: {})",
        agent_id, service_id
    ));

    Ok(())
}

async fn restart_mcp(ctx: &Arc<AppContext>, server_name: &str) -> Result<()> {
    CliService::section(&format!("Restarting MCP Server: {}", server_name));

    let manager = McpManager::new(ctx.clone())
        .await
        .context("Failed to initialize MCP manager")?;

    manager
        .restart_services(Some(server_name.to_string()))
        .await?;

    CliService::success(&format!(
        "✅ MCP server {} restarted successfully",
        server_name
    ));

    Ok(())
}

async fn restart_failed_services(ctx: &Arc<AppContext>) -> Result<()> {
    CliService::section("Restarting Failed Services");

    let mut restarted_count = 0;
    let mut failed_count = 0;

    let orchestrator = AgentOrchestrator::new(ctx.clone())
        .await
        .context("Failed to initialize agent orchestrator")?;

    let agent_registry = AgentRegistry::new().await?;

    let all_agents = orchestrator.list_all().await?;
    for (agent_id, status) in &all_agents {
        let agent_config = match agent_registry.get_agent(agent_id).await {
            Ok(config) => config,
            Err(_) => continue,
        };

        if !agent_config.enabled {
            continue;
        }

        if let systemprompt_core_agent::services::agent_orchestration::AgentStatus::Failed {
            ..
        } = status
        {
            CliService::info(&format!(
                "🔄 Restarting failed agent: {}",
                agent_config.name
            ));
            match orchestrator.restart_agent(agent_id).await {
                Ok(_) => {
                    restarted_count += 1;
                    CliService::success(&format!("  ✅ {} restarted", agent_config.name));
                },
                Err(e) => {
                    failed_count += 1;
                    CliService::error(&format!(
                        "  ❌ Failed to restart {}: {}",
                        agent_config.name, e
                    ));
                },
            }
        }
    }

    let mcp_manager = McpManager::new(ctx.clone())
        .await
        .context("Failed to initialize MCP manager")?;

    let registry = systemprompt_core_mcp::services::RegistryManager::new().await?;
    let servers = registry.get_enabled_servers().await?;

    for server in servers {
        if !server.enabled {
            continue;
        }

        let database = systemprompt_core_mcp::services::DatabaseManager::new(ctx.db_pool().clone());
        let service_info = database.get_service_by_name(&server.name).await?;

        let needs_restart = match service_info {
            Some(info) => info.status != "running",
            None => true,
        };

        if needs_restart {
            CliService::info(&format!("🔄 Restarting MCP server: {}", server.name));
            match mcp_manager
                .restart_services(Some(server.name.clone()))
                .await
            {
                Ok(_) => {
                    restarted_count += 1;
                    CliService::success(&format!("  ✅ {} restarted", server.name));
                },
                Err(e) => {
                    failed_count += 1;
                    CliService::error(&format!("  ❌ Failed to restart {}: {}", server.name, e));
                },
            }
        }
    }

    println!();
    if restarted_count > 0 {
        CliService::success(&format!("✅ Restarted {} failed services", restarted_count));
    } else {
        CliService::info("ℹ  No failed services found");
    }

    if failed_count > 0 {
        CliService::warning(&format!("⚠️  Failed to restart {} services", failed_count));
    }

    Ok(())
}
