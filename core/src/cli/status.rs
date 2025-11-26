use anyhow::{Context, Result};
use std::sync::Arc;
use systemprompt_core_agent::services::agent_orchestration::AgentOrchestrator;
use systemprompt_core_logging::CliService;
use systemprompt_core_mcp::services::McpManager;
use systemprompt_core_system::AppContext;

pub async fn execute() -> Result<()> {
    let ctx = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );

    CliService::section("SystemPrompt Services Status");
    println!();

    display_api_status(&ctx).await?;
    println!();

    display_agent_status(&ctx).await?;
    println!();

    display_mcp_status(&ctx).await?;

    Ok(())
}

async fn display_api_status(_ctx: &Arc<AppContext>) -> Result<()> {
    CliService::section("API Server");
    CliService::info("Use 'curl http://localhost:8080/api/v1/health' to check API server status");
    Ok(())
}

async fn display_agent_status(ctx: &Arc<AppContext>) -> Result<()> {
    let orchestrator = AgentOrchestrator::new(ctx.clone())
        .await
        .context("Failed to initialize agent orchestrator")?;

    orchestrator.show_detailed_status().await?;
    Ok(())
}

async fn display_mcp_status(ctx: &Arc<AppContext>) -> Result<()> {
    let manager = McpManager::new(ctx.clone())
        .await
        .context("Failed to initialize MCP manager")?;

    manager.show_status().await?;
    Ok(())
}
