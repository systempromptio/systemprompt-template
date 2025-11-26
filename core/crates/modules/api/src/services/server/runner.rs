use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_logging::CliService;
use systemprompt_core_scheduler::models::SchedulerConfig;
use systemprompt_core_scheduler::services::SchedulerService;
use systemprompt_core_system::AppContext;
use uuid::Uuid;

pub async fn run_server(ctx: AppContext) -> Result<()> {
    let mcp_orchestrator = create_mcp_orchestrator(&ctx).await?;

    reconcile_system_services(&ctx, &mcp_orchestrator).await?;
    spawn_mcp_monitor(&ctx, mcp_orchestrator).await?;

    let api_server = crate::services::server::setup_api_server(&ctx).await?;
    let addr = ctx.server_address();

    // Spawn agent reconciliation in background after API server is ready
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        // Wait for API server to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        match reconcile_agents(&ctx_clone).await {
            Ok(started_count) => {
                if started_count > 0 {
                    CliService::success(&format!("✅ Started {} enabled agents", started_count));
                }
            },
            Err(e) => {
                CliService::error(&format!("❌ FATAL: Agent reconciliation failed: {}", e));
                std::process::exit(1);
            },
        }
    });

    // Spawn scheduler initialization in background after API server is ready
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        // Wait for API server to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        match initialize_scheduler(&ctx_clone).await {
            Ok(_) => {
                CliService::success("✅ Scheduler initialized and started");
            },
            Err(e) => {
                CliService::warning(&format!("⚠️  Scheduler initialization failed: {}", e));
            },
        }
    });

    api_server.serve(&addr).await
}

async fn create_mcp_orchestrator(
    ctx: &AppContext,
) -> Result<Arc<systemprompt_core_mcp::services::McpManager>> {
    use systemprompt_core_mcp::services::McpManager;
    let manager = McpManager::new(Arc::new(ctx.clone())).await?;
    Ok(Arc::new(manager))
}

async fn cleanup_stale_service_entries(ctx: &AppContext) -> Result<u64> {
    use systemprompt_core_logging::CliService;
    use systemprompt_models::repository::ServiceRepository;

    let repo = ServiceRepository::new(ctx.db_pool().clone());
    let mut deleted_count = 0u64;

    let mcp_services = repo.get_mcp_services().await.unwrap_or_default();
    for service in mcp_services {
        let should_delete = if service.status == "running" {
            if let Some(pid) = service.pid {
                let process_exists = std::path::Path::new(&format!("/proc/{}", pid)).exists();

                let port_responsive = {
                    use tokio::net::TcpStream;
                    use tokio::time::{timeout, Duration};
                    timeout(
                        Duration::from_secs(2),
                        TcpStream::connect(format!("127.0.0.1:{}", service.port)),
                    )
                    .await
                    .is_ok()
                };

                !process_exists || !port_responsive
            } else {
                true
            }
        } else {
            service.status == "error" || service.status == "stopped"
        };

        if should_delete {
            match repo.delete_service(&service.name).await {
                Ok(_) => {
                    deleted_count += 1;
                    CliService::info(&format!(
                        "   Deleted stale MCP service '{}' (status: {}, pid: {:?})",
                        service.name, service.status, service.pid
                    ));
                },
                Err(e) => {
                    CliService::warning(&format!(
                        "⚠️  Failed to delete MCP service '{}': {}",
                        service.name, e
                    ));
                },
            }
        }
    }

    let agent_service_names = repo.get_all_agent_service_names().await.unwrap_or_default();
    for service_name in agent_service_names {
        if let Ok(Some(service)) = repo.get_service_by_name(&service_name).await {
            let should_delete = if service.status == "running" {
                if let Some(pid) = service.pid {
                    let process_exists = std::path::Path::new(&format!("/proc/{}", pid)).exists();

                    let port_responsive = {
                        use tokio::net::TcpStream;
                        use tokio::time::{timeout, Duration};
                        timeout(
                            Duration::from_secs(2),
                            TcpStream::connect(format!("127.0.0.1:{}", service.port)),
                        )
                        .await
                        .is_ok()
                    };

                    !process_exists || !port_responsive
                } else {
                    true
                }
            } else {
                service.status == "error" || service.status == "stopped"
            };

            if should_delete {
                match repo.delete_service(&service_name).await {
                    Ok(_) => {
                        deleted_count += 1;
                        CliService::info(&format!(
                            "   Deleted stale agent service '{}' (status: {}, pid: {:?})",
                            service_name, service.status, service.pid
                        ));
                    },
                    Err(e) => {
                        CliService::warning(&format!(
                            "⚠️  Failed to delete agent service '{}': {}",
                            service_name, e
                        ));
                    },
                }
            }
        }
    }

    Ok(deleted_count)
}

async fn reconcile_system_services(
    ctx: &AppContext,
    mcp_orchestrator: &Arc<systemprompt_core_mcp::services::McpManager>,
) -> Result<()> {
    CliService::section("System Services Reconciliation");

    // Clean stale entries from shared database before reconciliation
    CliService::info("🧹 Cleaning stale service entries from database...");
    match cleanup_stale_service_entries(ctx).await {
        Ok(count) => {
            if count > 0 {
                CliService::success(&format!("✅ Cleaned {} stale service entries", count));
            } else {
                CliService::info("   No stale entries found");
            }
        },
        Err(e) => {
            CliService::warning(&format!("⚠️  Could not clean stale entries: {}", e));
        },
    }

    let required_servers = mcp_orchestrator.get_enabled_servers().await?;
    let required_count = required_servers.len();

    match mcp_orchestrator.reconcile().await {
        Ok(running_count) => {
            let required_names: Vec<String> =
                required_servers.iter().map(|s| s.name.clone()).collect();

            CliService::info(&format!(
                "📊 MCP Server Status: {} required, {} running",
                required_count, running_count
            ));
            CliService::info(&format!("   Required: {}", required_names.join(", ")));

            if running_count < required_count {
                let running_servers = mcp_orchestrator.get_running_servers().await?;
                let running_names: std::collections::HashSet<String> =
                    running_servers.iter().map(|s| s.name.clone()).collect();
                let running_list: Vec<String> =
                    running_servers.iter().map(|s| s.name.clone()).collect();

                CliService::info(&format!("   Running: {}", running_list.join(", ")));

                let missing: Vec<String> = required_servers
                    .iter()
                    .map(|s| s.name.clone())
                    .filter(|name| !running_names.contains(name))
                    .collect();

                CliService::error(&format!(
                    "❌ Server status mismatch: {} servers failed to start: {}",
                    missing.len(),
                    missing.join(", ")
                ));

                panic!(
                    "❌ FATAL: {} required MCP server(s) failed to start: {}\n\n\
                    SystemPrompt OS cannot operate without MCP servers.\n\
                    Agents need tools to function.\n\n\
                    Build missing binaries with:\n  \
                    cargo build --bin {}\n\n\
                    Or build all MCP servers:\n  \
                    just mcp build",
                    missing.len(),
                    missing.join(", "),
                    missing.join(" --bin ")
                );
            } else {
                let running_servers = mcp_orchestrator.get_running_servers().await?;
                let running_list: Vec<String> =
                    running_servers.iter().map(|s| s.name.clone()).collect();
                CliService::info(&format!("   Running: {}", running_list.join(", ")));
            }

            if running_count > 0 {
                use systemprompt_models::repository::ServiceRepository;
                let service_repo = ServiceRepository::new(ctx.db_pool().clone());

                CliService::info("🔍 Verifying database registration for all services...");
                let mut verification_failed = Vec::new();

                for server in &required_servers {
                    match service_repo.get_service_by_name(&server.name).await {
                        Ok(Some(service)) if service.status == "running" => {
                            CliService::info(&format!(
                                "   ✓ {} registered (pid: {:?})",
                                server.name, service.pid
                            ));
                        },
                        Ok(Some(service)) => {
                            verification_failed
                                .push(format!("{} (status: {})", server.name, service.status));
                        },
                        Ok(None) => {
                            verification_failed.push(format!("{} (not in database)", server.name));
                        },
                        Err(e) => {
                            verification_failed.push(format!("{} (db error: {})", server.name, e));
                        },
                    }
                }

                if !verification_failed.is_empty() {
                    CliService::error(&format!(
                        "❌ Database verification failed for {} service(s): {}",
                        verification_failed.len(),
                        verification_failed.join(", ")
                    ));
                    panic!(
                        "❌ FATAL: MCP services running but not properly registered in database\n\n\
                        This indicates a race condition or database synchronization issue.\n\
                        Failed services: {}",
                        verification_failed.join(", ")
                    );
                }

                CliService::success(&format!(
                    "✅ All {} required MCP servers are running and verified in database",
                    running_count
                ));
            }
        },
        Err(e) => {
            CliService::error(&format!("   ❌ MCP reconciliation error: {:?}", e));
            panic!(
                "❌ FATAL: MCP reconciliation failed: {}\n\nCannot start API without MCP servers.",
                e
            );
        },
    }

    // Skip agent reconciliation during startup - agents will be started after API server is listening
    // This avoids chicken-and-egg problem where agents need API server for health checks
    CliService::info(
        "⏭️  Skipping agent reconciliation during startup (will start after API is ready)",
    );

    CliService::success("✅ System services reconciliation complete");
    Ok(())
}

async fn reconcile_agents(ctx: &AppContext) -> Result<usize> {
    use systemprompt_core_agent::services::agent_orchestration::AgentOrchestrator;
    use systemprompt_core_agent::services::registry::AgentRegistry;

    CliService::info("🤖 Initializing agent orchestrator...");
    let orchestrator = match AgentOrchestrator::new(Arc::new(ctx.clone())).await {
        Ok(orch) => {
            CliService::success("   ✅ Agent orchestrator ready");
            orch
        },
        Err(e) => {
            CliService::error(&format!("   ❌ Failed to initialize orchestrator: {}", e));
            return Err(e.into());
        },
    };

    CliService::info("📋 Loading agent registry...");
    let agent_registry = match AgentRegistry::new().await {
        Ok(registry) => {
            CliService::success("   ✅ Agent registry loaded");
            registry
        },
        Err(e) => {
            CliService::error(&format!("   ❌ Failed to load registry: {}", e));
            return Err(e.into());
        },
    };

    CliService::info("📊 Fetching enabled agents...");
    let enabled_agents = match agent_registry.list_enabled_agents().await {
        Ok(agents) => {
            CliService::success(&format!("   ✅ Found {} enabled agent(s)", agents.len()));
            agents
        },
        Err(e) => {
            CliService::error(&format!("   ❌ Failed to list enabled agents: {}", e));
            return Err(e);
        },
    };

    let required_count = enabled_agents.len();

    let mut started = 0;
    let mut failed_agents: Vec<(String, String)> = Vec::new();

    for agent_config in enabled_agents {
        let desired_name = &agent_config.name;
        let desired_port = agent_config.port;

        CliService::info(&format!(
            "🚀 Starting agent '{}' (port {})...",
            desired_name, desired_port
        ));

        match enforce_clean_agent_state(&orchestrator, desired_name, desired_port).await {
            Ok(true) => {
                CliService::success(&format!(
                    "   ✅ Successfully started agent '{}'",
                    desired_name
                ));
                started += 1;
            },
            Ok(false) => {
                CliService::info(&format!(
                    "   ℹ️  Agent '{}' was already running",
                    desired_name
                ));
                started += 1;
            },
            Err(e) => {
                CliService::error(&format!(
                    "   ❌ Failed to reconcile agent '{}': {}",
                    desired_name, e
                ));
                failed_agents.push((desired_name.clone(), e.to_string()));
            },
        }
    }

    if !failed_agents.is_empty() {
        CliService::warning(&format!(
            "⚠️  {} agent(s) failed to start on first attempt",
            failed_agents.len()
        ));

        for (name, err) in &failed_agents {
            CliService::warning(&format!("   • {}: {}", name, err));
        }

        CliService::info("🔄 Attempting cleanup and retry...");

        let mut retry_failed: Vec<(String, String)> = Vec::new();

        for (agent_name, original_error) in &failed_agents {
            CliService::info(&format!(
                "🔄 Retrying agent '{}' after initial failure...",
                agent_name
            ));
            CliService::debug(&format!("   Original error: {}", original_error));

            let agent_config = match agent_registry.get_agent(agent_name).await {
                Ok(config) => {
                    CliService::debug(&format!(
                        "   ✓ Agent config retrieved (port {})",
                        config.port
                    ));
                    config
                },
                Err(e) => {
                    CliService::error(&format!(
                        "   ✗ Failed to retrieve config for '{}': {}",
                        agent_name, e
                    ));
                    retry_failed
                        .push((agent_name.clone(), format!("Agent config not found: {}", e)));
                    continue;
                },
            };

            match enforce_clean_agent_state(&orchestrator, agent_name, agent_config.port).await {
                Ok(_) => {
                    CliService::success(&format!("   ✅ Retry successful for {}", agent_name));
                    started += 1;
                },
                Err(e) => {
                    CliService::error(&format!("   ✗ Retry failed for '{}': {}", agent_name, e));
                    retry_failed.push((agent_name.clone(), e.to_string()));
                },
            }
        }

        if !retry_failed.is_empty() {
            let agent_names: Vec<String> =
                retry_failed.iter().map(|(name, _)| name.clone()).collect();
            panic!(
                "❌ FATAL: {} required agent(s) failed to start after retry: {}\n\n\
                SystemPrompt OS cannot operate without all enabled agents.\n\
                Agents are the core service layer.\n\n\
                Failures:\n{}\n\n\
                Possible causes:\n\
                  • Agent binaries not built (run: cargo build)\n\
                  • Ports occupied by non-agent processes (check with: lsof -i:PORT)\n\
                  • Missing environment variables (check .env file)\n\
                  • File permission issues\n\n\
                Build agents with: cargo build",
                retry_failed.len(),
                agent_names.join(", "),
                retry_failed
                    .iter()
                    .map(|(name, err)| format!("  • {}: {}", name, err))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }

        CliService::success("✅ All agents started successfully after retry");
    }

    if started < required_count {
        panic!(
            "❌ FATAL: Only {}/{} required agents started successfully\n\n\
            All enabled agents must be running for API to start.",
            started, required_count
        );
    }

    if started > 0 {
        CliService::success(&format!("✅ All {} required agents are running", started));
    }

    Ok(started)
}

async fn enforce_clean_agent_state(
    orchestrator: &systemprompt_core_agent::services::agent_orchestration::AgentOrchestrator,
    agent_id: &str,
    desired_port: u16,
) -> Result<bool> {
    use systemprompt_core_agent::services::agent_orchestration::{AgentStatus, PortManager};

    match orchestrator.get_status(agent_id).await {
        Ok(status) => match status {
            AgentStatus::Running { pid, port } => {
                if port == desired_port {
                    use systemprompt_core_agent::services::agent_orchestration::process;
                    if process::process_exists(pid) {
                        return Ok(false);
                    } else {
                        CliService::info(&format!(
                            "   Agent {} process {} died, restarting...",
                            agent_id, pid
                        ));
                    }
                } else {
                    use systemprompt_core_agent::services::agent_orchestration::process;
                    CliService::info(&format!(
                        "   Agent {} on wrong port {} (expected {}), killing and restarting...",
                        agent_id, port, desired_port
                    ));
                    if process::kill_process(pid) {
                        CliService::success(&format!("   Killed wrong-port process {}", pid));
                    }
                    orchestrator.delete_agent(agent_id).await.ok();
                }
            },
            AgentStatus::Failed { .. } => {
                CliService::info(&format!(
                    "   Agent {} previously failed, restarting...",
                    agent_id
                ));
            },
        },
        Err(_) => {},
    }

    let port_manager = PortManager::new();
    if let Err(e) = port_manager.cleanup_port_if_needed(desired_port).await {
        CliService::error(&format!(
            "   Failed to cleanup port {} for agent {}: {}",
            desired_port, agent_id, e
        ));
        return Err(e.into());
    }

    match orchestrator.start_agent(agent_id).await {
        Ok(_) => {
            CliService::success(&format!(
                "   Started agent {} on port {}",
                agent_id, desired_port
            ));
            Ok(true)
        },
        Err(e) => Err(e.into()),
    }
}

async fn spawn_mcp_monitor(
    ctx: &AppContext,
    _orchestrator: Arc<systemprompt_core_mcp::services::McpManager>,
) -> Result<()> {
    let db_pool = ctx.db_pool().clone();

    tokio::spawn(async move {
        if let Err(e) =
            systemprompt_core_mcp::services::process::monitor::monitor_processes(db_pool, ()).await
        {
            CliService::error(&format!("❌ MCP monitor exited with error: {}", e));
        }
    });

    CliService::success("✅ MCP process monitor started");
    Ok(())
}

async fn initialize_scheduler(ctx: &AppContext) -> Result<()> {
    use systemprompt_core_logging::LogContext;
    use systemprompt_core_scheduler::repository::SchedulerRepository;
    use systemprompt_core_scheduler::services::jobs;

    CliService::info("🕐 Initializing scheduler...");

    let scheduler = SchedulerService::new(
        SchedulerConfig::default(),
        ctx.db_pool().clone(),
        Arc::new(ctx.clone()),
    );

    scheduler.start().await?;

    // Run cleanup jobs immediately on bootstrap
    CliService::info("🧹 Running cleanup jobs on bootstrap...");

    let db_pool = ctx.db_pool().clone();
    let scheduler_repo = SchedulerRepository::new(db_pool.clone());
    let log_context = LogContext::new()
        .with_session_id("scheduler-bootstrap")
        .with_trace_id(&format!("bootstrap-{}", Uuid::new_v4()))
        .with_user_id("system");

    let logger = systemprompt_core_logging::LogService::new(db_pool.clone(), log_context);

    // Database cleanup
    scheduler_repo
        .increment_run_count("database_cleanup")
        .await
        .ok();
    match jobs::database_cleanup(db_pool.clone(), logger.clone(), Arc::new(ctx.clone())).await {
        Ok(_) => {
            CliService::success("✅ Database cleanup job completed");
            scheduler_repo
                .update_job_execution("database_cleanup", "success", None, None)
                .await
                .ok();
        },
        Err(e) => {
            CliService::warning(&format!("⚠️  Database cleanup job failed: {}", e));
            scheduler_repo
                .update_job_execution("database_cleanup", "failed", Some(&e.to_string()), None)
                .await
                .ok();
        },
    }

    // Inactive sessions cleanup
    scheduler_repo
        .increment_run_count("cleanup_inactive_sessions")
        .await
        .ok();
    match jobs::cleanup_inactive_sessions(db_pool.clone(), logger.clone(), Arc::new(ctx.clone()))
        .await
    {
        Ok(_) => {
            CliService::success("✅ Inactive sessions cleanup job completed");
            scheduler_repo
                .update_job_execution("cleanup_inactive_sessions", "success", None, None)
                .await
                .ok();
        },
        Err(e) => {
            CliService::warning(&format!("⚠️  Inactive sessions cleanup job failed: {}", e));
            scheduler_repo
                .update_job_execution(
                    "cleanup_inactive_sessions",
                    "failed",
                    Some(&e.to_string()),
                    None,
                )
                .await
                .ok();
        },
    }

    // Anonymous users cleanup
    scheduler_repo
        .increment_run_count("cleanup_anonymous_users")
        .await
        .ok();
    match jobs::cleanup_anonymous_users(db_pool.clone(), logger.clone(), Arc::new(ctx.clone()))
        .await
    {
        Ok(_) => {
            CliService::success("✅ Anonymous users cleanup job completed");
            scheduler_repo
                .update_job_execution("cleanup_anonymous_users", "success", None, None)
                .await
                .ok();
        },
        Err(e) => {
            CliService::warning(&format!("⚠️  Anonymous users cleanup job failed: {}", e));
            scheduler_repo
                .update_job_execution(
                    "cleanup_anonymous_users",
                    "failed",
                    Some(&e.to_string()),
                    None,
                )
                .await
                .ok();
        },
    }

    // Content ingestion
    scheduler_repo
        .increment_run_count("content_ingestion")
        .await
        .ok();
    match jobs::ingest_content(db_pool.clone(), logger.clone(), Arc::new(ctx.clone())).await {
        Ok(_) => {
            CliService::success("✅ Content ingestion job completed");
            scheduler_repo
                .update_job_execution("content_ingestion", "success", None, None)
                .await
                .ok();

            // Wait for database transaction to be fully visible (Docker race condition fix)
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        },
        Err(e) => {
            CliService::warning(&format!("⚠️  Content ingestion job failed: {}", e));
            scheduler_repo
                .update_job_execution("content_ingestion", "failed", Some(&e.to_string()), None)
                .await
                .ok();
        },
    }

    // Regenerate static content (prerender + sitemap)
    scheduler_repo
        .increment_run_count("regenerate_static_content")
        .await
        .ok();
    match jobs::regenerate_static_content(db_pool.clone(), logger.clone(), Arc::new(ctx.clone()))
        .await
    {
        Ok(_) => {
            CliService::success("✅ Static content regeneration job completed");
            scheduler_repo
                .update_job_execution("regenerate_static_content", "success", None, None)
                .await
                .ok();
        },
        Err(e) => {
            CliService::warning(&format!(
                "⚠️  Static content regeneration job failed: {}",
                e
            ));
            scheduler_repo
                .update_job_execution(
                    "regenerate_static_content",
                    "failed",
                    Some(&e.to_string()),
                    None,
                )
                .await
                .ok();
        },
    }

    Ok(())
}
