//! Main Agent Orchestrator - Clean Service-Driven Implementation

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use systemprompt_core_logging::CliService;
use systemprompt_core_system::AppContext;
use tokio::task::JoinHandle;

use crate::services::agent_orchestration::database::AgentDatabaseService;
use crate::services::agent_orchestration::lifecycle::AgentLifecycle;
use crate::services::agent_orchestration::monitor::AgentMonitor;
use crate::services::agent_orchestration::reconciler::AgentReconciler;
use crate::services::agent_orchestration::{
    monitor, AgentStatus, OrchestrationError, OrchestrationResult, ValidationReport,
};

#[derive(Debug)]
struct AgentInfo {
    id: String,
    name: String,
    status: AgentStatus,
    port: u16,
}

#[derive(Debug)]
pub struct AgentOrchestrator {
    db_service: AgentDatabaseService,
    lifecycle: AgentLifecycle,
    reconciler: AgentReconciler,
    monitor: AgentMonitor,
    monitoring_handle: Option<JoinHandle<Result<()>>>,
    ctx: Arc<AppContext>,
}

impl AgentOrchestrator {
    pub async fn new(ctx: Arc<AppContext>) -> OrchestrationResult<Self> {
        CliService::info("🎭 Initializing Agent Orchestrator...");

        let db_pool = ctx.db_pool();

        use crate::repository::AgentServiceRepository;
        let agent_repo = AgentServiceRepository::new(db_pool.clone());

        let db_service = AgentDatabaseService::new(agent_repo).await?;
        let lifecycle = AgentLifecycle::new(db_pool.clone()).await?;
        let reconciler = AgentReconciler::new(db_pool.clone()).await?;
        let monitor = AgentMonitor::new(db_pool.clone()).await?;

        let orchestrator = Self {
            db_service,
            lifecycle,
            reconciler,
            monitor,
            monitoring_handle: None,
            ctx,
        };

        orchestrator.startup_reconciliation().await?;

        // Only start health monitoring in daemon mode, not for CLI commands
        // This prevents background tasks from preventing clean exit
        // orchestrator.start_health_monitoring();

        CliService::success("✅ Agent Orchestrator initialized");
        Ok(orchestrator)
    }

    pub async fn start_agent(&self, agent_id: &str) -> OrchestrationResult<String> {
        self.lifecycle.start_agent(agent_id).await
    }

    pub async fn enable_agent(&self, agent_id: &str) -> OrchestrationResult<String> {
        self.lifecycle.enable_agent(agent_id).await
    }

    pub async fn disable_agent(&self, agent_id: &str) -> OrchestrationResult<()> {
        self.lifecycle.disable_agent(agent_id).await
    }

    pub async fn restart_agent(&self, agent_id: &str) -> OrchestrationResult<String> {
        self.lifecycle.restart_agent(agent_id).await
    }

    pub async fn get_status(&self, agent_id: &str) -> OrchestrationResult<AgentStatus> {
        self.db_service.get_status(agent_id).await
    }

    pub async fn list_agents(&self) -> OrchestrationResult<Vec<(String, AgentStatus)>> {
        self.db_service.list_all_agents().await
    }

    pub async fn cleanup_crashed_agents(&self) -> OrchestrationResult<u64> {
        self.db_service.cleanup_orphaned_services().await
    }

    pub async fn health_check(
        &self,
        agent_id: &str,
    ) -> OrchestrationResult<monitor::HealthCheckResult> {
        self.monitor.comprehensive_health_check(agent_id).await
    }

    pub async fn start_all(&self) -> OrchestrationResult<Vec<String>> {
        let agents = self.db_service.list_all_agents().await?;
        let mut service_ids = Vec::new();

        for (agent_id, status) in agents {
            if matches!(status, AgentStatus::Failed { .. }) {
                match self.start_agent(&agent_id).await {
                    Ok(service_id) => service_ids.push(service_id),
                    Err(e) => CliService::error(&format!("Failed to start {agent_id}: {e}")),
                }
            }
        }

        Ok(service_ids)
    }

    pub async fn disable_all(&self) -> OrchestrationResult<()> {
        let agents = self.db_service.list_all_agents().await?;

        for (agent_id, _) in agents {
            if let Err(e) = self.disable_agent(&agent_id).await {
                CliService::error(&format!("Failed to disable {agent_id}: {e}"));
            }
        }

        Ok(())
    }

    pub async fn show_detailed_status(&self) -> OrchestrationResult<()> {
        let agent_info = self.get_comprehensive_agent_info().await?;

        CliService::info("📊 Agent Status Report:");
        CliService::info("");

        // Print table header
        CliService::info(&format!(
            "{:<32} {:<20} {:<8} {:<8} {:<6} {:<40}",
            "ID", "NAME", "STATUS", "PID", "PORT", "URL"
        ));
        CliService::info(&format!(
            "{:<32} {:<20} {:<8} {:<8} {:<6} {:<40}",
            "━".repeat(32),
            "━".repeat(20),
            "━".repeat(8),
            "━".repeat(8),
            "━".repeat(6),
            "━".repeat(40)
        ));

        // Print agent data
        for info in agent_info {
            let (status_str, status_detail) = match &info.status {
                AgentStatus::Running { .. } => ("Running".to_string(), None),
                AgentStatus::Failed { reason, .. } => {
                    let short_reason = if reason.len() > 40 {
                        format!("{}...", &reason[..37])
                    } else {
                        reason.clone()
                    };
                    ("Failed".to_string(), Some(short_reason))
                },
            };

            // Get API base URL from app context
            let api_base = format!("http://{}", self.ctx.server_address());

            let (pid_str, port_str) = match info.status {
                AgentStatus::Running { pid, port } => (pid.to_string(), port.to_string()),
                AgentStatus::Failed { .. } => ("-".to_string(), info.port.to_string()),
            };

            let url = format!("{}/api/v1/agents/{}", api_base, info.id);

            CliService::info(&format!(
                "{:<32} {:<20} {:<8} {:<8} {:<6} {:<40}",
                &info.id[..std::cmp::min(32, info.id.len())],
                &info.name[..std::cmp::min(20, info.name.len())],
                status_str,
                pid_str,
                port_str,
                url
            ));

            if let Some(detail) = status_detail {
                CliService::info(&format!("   └─ Reason: {detail}"));
            }
        }

        Ok(())
    }

    async fn get_comprehensive_agent_info(&self) -> OrchestrationResult<Vec<AgentInfo>> {
        let agents = self.db_service.list_all_agents().await?;
        let mut agent_info = Vec::new();

        for (agent_id, status) in agents {
            // Get agent name and port from database
            match self.db_service.get_agent_config(&agent_id).await {
                Ok(config) => {
                    agent_info.push(AgentInfo {
                        id: agent_id,
                        name: config.name,
                        status,
                        port: config.port,
                    });
                },
                Err(_) => {
                    // If we can't get config, use defaults
                    let port = match status {
                        AgentStatus::Running { port, .. } => port,
                        _ => 8000, // default port
                    };
                    agent_info.push(AgentInfo {
                        id: agent_id.clone(),
                        name: "Unknown".to_string(),
                        status,
                        port,
                    });
                },
            }
        }

        Ok(agent_info)
    }

    pub async fn list_all(&self) -> OrchestrationResult<Vec<(String, AgentStatus)>> {
        self.db_service.list_all_agents().await
    }

    pub async fn validate_agent(&self, agent_id: &str) -> OrchestrationResult<ValidationReport> {
        let mut report = ValidationReport::new();

        // Check if agent exists
        let exists = self.db_service.agent_exists(agent_id).await?;
        if !exists {
            report.add_issue("Agent not found in database".to_string());
            return Ok(report);
        }

        // Check agent configuration
        match self.db_service.get_agent_config(agent_id).await {
            Ok(_) => {}, // Config is valid
            Err(e) => {
                report.add_issue(format!("Configuration error: {e}"));
                return Ok(report);
            },
        }

        // Check agent status
        let status = self.db_service.get_status(agent_id).await?;
        match status {
            AgentStatus::Running { .. } => {
                // Try health check for running agents
                match self.health_check(agent_id).await {
                    Ok(health) => {
                        if !health.healthy {
                            report.add_issue(format!("Health check failed: {}", health.message));
                        }
                    },
                    Err(e) => {
                        report.add_issue(format!("Health check error: {e}"));
                    },
                }
            },
            AgentStatus::Failed { reason, .. } => {
                report.add_issue(format!("Agent is in failed state: {reason}"));
            },
        }

        Ok(report)
    }

    pub async fn health_check_all(&self) -> OrchestrationResult<Vec<monitor::HealthCheckResult>> {
        let running_agents = self.db_service.list_running_agents().await?;
        let mut results = Vec::new();

        for agent_id in running_agents {
            match self.health_check(&agent_id).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    CliService::warning(&format!("Health check failed for {agent_id}: {e}"));
                    // Continue with other agents even if one fails
                },
            }
        }

        Ok(results)
    }

    pub async fn run_daemon(&mut self) -> OrchestrationResult<()> {
        CliService::info("🚀 Starting Agent Orchestrator daemon...");

        // Start health monitoring in daemon mode
        self.start_health_monitoring();

        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    CliService::info("Received shutdown signal");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_secs(60)) => {
                    if let Err(e) = self.cleanup_crashed_agents().await {
                        CliService::error(&format!("Cleanup error: {e}"));
                    }
                }
            }
        }

        self.shutdown().await;
        Ok(())
    }

    async fn startup_reconciliation(&self) -> OrchestrationResult<()> {
        CliService::info("🔄 Performing startup reconciliation...");

        let reconciled = self.reconciler.reconcile_running_services().await?;
        let started_fixed = self.reconciler.reconcile_starting_services().await?;

        let report = self.reconciler.perform_consistency_check().await?;
        if report.has_inconsistencies() {
            let fixed = self.reconciler.fix_inconsistencies(&report).await?;
            CliService::success(&format!("🔧 Fixed {} inconsistencies", fixed));
        }

        let total_fixed = reconciled + started_fixed;
        if total_fixed > 0 {
            CliService::success(&format!(
                "✅ Startup reconciliation complete - fixed {} issues",
                total_fixed
            ));
        } else {
            CliService::success("✅ Startup reconciliation complete - no issues found");
        }

        Ok(())
    }

    fn start_health_monitoring(&mut self) {
        let pool = self.ctx.db_pool().clone();

        let handle = tokio::spawn(async move {
            let monitor = match AgentMonitor::new(pool).await {
                Ok(m) => m,
                Err(e) => {
                    CliService::error(&format!("❌ Failed to initialize health monitor: {e}"));
                    return Ok(());
                },
            };

            let mut interval = tokio::time::interval(Duration::from_secs(60));

            // Skip the first immediate tick and wait for the actual interval
            interval.tick().await;

            loop {
                interval.tick().await;

                match monitor.monitor_all_agents().await {
                    Ok(report) => {
                        if report.total_agents() > 0 {
                            CliService::info(&format!(
                                "💓 Health check: {}/{} agents healthy ({:.1}%)",
                                report.healthy_agents.len(),
                                report.total_agents(),
                                report.healthy_percentage()
                            ));
                        }
                    },
                    Err(e) => {
                        CliService::error(&format!("❌ Health monitoring error: {e}"));
                    },
                }

                if let Err(e) = monitor.cleanup_unresponsive_agents(3).await {
                    CliService::error(&format!("❌ Cleanup error: {e}"));
                }
            }
        });

        self.monitoring_handle = Some(handle);
    }

    pub async fn shutdown(&mut self) {
        CliService::info("🛑 Shutting down Agent Orchestrator...");

        if let Some(handle) = self.monitoring_handle.take() {
            handle.abort();
            CliService::info("   Stopped health monitoring");
        }

        CliService::success("✅ Agent Orchestrator shutdown complete");
    }

    pub async fn delete_agent(&self, agent_id: &str) -> OrchestrationResult<()> {
        CliService::info(&format!("🗑️ Deleting agent: {agent_id}"));

        // Stop agent if it's running
        if let Ok(status) = self.get_status(agent_id).await {
            if let AgentStatus::Running { .. } = status {
                CliService::info("🛑 Stopping running agent before deletion...");
                self.lifecycle.disable_agent(agent_id).await?;
            }
        }

        // Remove from services table
        self.db_service.remove_agent_service(agent_id).await?;

        // Note: Agent config deletion must be done by removing from YAML config files
        // The database only tracks runtime service state, not configuration

        CliService::success(&format!("✅ Agent {} deleted successfully", agent_id));
        Ok(())
    }

    pub async fn delete_all_agents(&self) -> OrchestrationResult<u64> {
        CliService::info("🗑️ Deleting all agents...");

        // Get all agents first
        let agents = self.list_all().await?;
        let total_count = agents.len() as u64;

        if total_count == 0 {
            CliService::info("✨ No agents to delete");
            return Ok(0);
        }

        CliService::info(&format!("🔍 Found {} agents to delete", total_count));

        // Disable all running agents first
        CliService::info("🛑 Disabling all running agents...");
        self.disable_all().await?;

        // Delete each agent
        let mut deleted_count = 0;
        for (agent_id, _) in agents {
            match self.delete_agent(&agent_id).await {
                Ok(_) => {
                    deleted_count += 1;
                },
                Err(e) => {
                    CliService::error(&format!("❌ Failed to delete agent {agent_id}: {e}"));
                },
            }
        }

        CliService::success(&format!(
            "✅ Deleted {} out of {} agents",
            deleted_count, total_count
        ));
        Ok(deleted_count)
    }

    pub async fn cleanup_orphaned_processes(&self) -> OrchestrationResult<()> {
        CliService::info("🧹 Scanning for orphaned agent processes...");

        // Find all agent-worker processes
        let output = std::process::Command::new("pgrep")
            .arg("-f")
            .arg("agent-worker")
            .output()
            .map_err(|e| {
                OrchestrationError::ProcessSpawnFailed(format!("Failed to run pgrep: {e}"))
            })?;

        if !output.status.success() {
            CliService::info("✨ No orphaned agent processes found");
            return Ok(());
        }

        let pids_str = String::from_utf8_lossy(&output.stdout);
        let mut registered = 0;
        let mut failed = 0;

        for line in pids_str.lines() {
            if let Ok(pid) = line.trim().parse::<u32>() {
                // Check if this PID is already tracked
                if self.is_pid_tracked(pid).await? {
                    continue;
                }

                // Try to determine which agent this process belongs to
                if let Some((agent_id, port)) = self.identify_orphaned_process(pid).await? {
                    CliService::info(&format!(
                        "🔍 Found orphaned process PID {} for agent {} on port {}",
                        pid, agent_id, port
                    ));

                    // Get agent name for orphaned process registration
                    let name = self
                        .db_service
                        .get_agent_config(&agent_id)
                        .await
                        .map(|config| config.name)
                        .unwrap_or_else(|_| "unknown".to_string());

                    // Register the process in services table with default auth='none' for orphaned
                    // processes
                    match self
                        .db_service
                        .register_agent(&name, pid, port, "none")
                        .await
                    {
                        Ok(service_id) => {
                            CliService::success(&format!(
                                "✅ Registered orphaned process as service: {}",
                                service_id
                            ));
                            registered += 1;
                        },
                        Err(e) => {
                            CliService::error(&format!(
                                "❌ Failed to register orphaned process: {}",
                                e
                            ));
                            failed += 1;
                        },
                    }
                } else {
                    CliService::warning(&format!("⚠️ Could not identify agent for PID {pid}"));
                    failed += 1;
                }
            }
        }

        if registered > 0 {
            CliService::success(&format!("✅ Registered {} orphaned processes", registered));
        }
        if failed > 0 {
            CliService::warning(&format!("⚠️ Failed to handle {} processes", failed));
        }
        if registered == 0 && failed == 0 {
            CliService::info("✨ No orphaned agent processes found");
        }

        Ok(())
    }

    async fn is_pid_tracked(&self, pid: u32) -> OrchestrationResult<bool> {
        let agents = self.db_service.list_all_agents().await?;
        for (_, status) in agents {
            if let AgentStatus::Running {
                pid: tracked_pid, ..
            } = status
            {
                if tracked_pid == pid {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    async fn identify_orphaned_process(
        &self,
        pid: u32,
    ) -> OrchestrationResult<Option<(String, u16)>> {
        // Try to read environment variables from /proc/{pid}/environ
        let environ_path = format!("/proc/{}/environ", pid);
        if let Ok(environ_data) = std::fs::read(&environ_path) {
            let environ_str = String::from_utf8_lossy(&environ_data);
            let mut agent_id = None;
            let mut port = None;

            // Parse environment variables (null-separated)
            for env_var in environ_str.split('\0') {
                if env_var.starts_with("AGENT_ID=") || env_var.starts_with("AGENT_UUID=") {
                    agent_id = env_var.split('=').nth(1).map(|s| s.to_string());
                } else if env_var.starts_with("AGENT_PORT=") {
                    if let Some(port_str) = env_var.split('=').nth(1) {
                        port = port_str.parse::<u16>().ok();
                    }
                }
            }

            if let (Some(id), Some(p)) = (agent_id, port) {
                return Ok(Some((id, p)));
            }
        }

        Ok(None)
    }
}

impl Drop for AgentOrchestrator {
    fn drop(&mut self) {
        if let Some(handle) = self.monitoring_handle.take() {
            handle.abort();
        }
    }
}
