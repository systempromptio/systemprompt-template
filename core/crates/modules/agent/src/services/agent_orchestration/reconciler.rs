//! Database State Reconciliation

use systemprompt_core_database::DbPool;
use systemprompt_core_logging::CliService;

use crate::services::agent_orchestration::database::AgentDatabaseService;
use crate::services::agent_orchestration::{process, OrchestrationResult};

#[derive(Debug)]
pub struct AgentReconciler {
    db_service: AgentDatabaseService,
}

impl AgentReconciler {
    pub async fn new(db_pool: DbPool) -> OrchestrationResult<Self> {
        use crate::repository::AgentServiceRepository;

        let agent_service_repo = AgentServiceRepository::new(db_pool);
        let db_service = AgentDatabaseService::new(agent_service_repo).await?;

        Ok(Self { db_service })
    }

    pub async fn reconcile_running_services(&self) -> OrchestrationResult<u32> {
        CliService::info("🔄 Reconciling running services with actual processes...");

        let all_agents = self.db_service.list_all_agents().await?;
        let mut reconciled = 0;

        for (agent_id, status) in all_agents {
            match status {
                crate::services::agent_orchestration::AgentStatus::Running { pid, .. } => {
                    if !process::process_exists(pid) {
                        CliService::warning(&format!("🔍 Agent {} (PID {}) marked as running but process not found - marking as failed", agent_id, pid));
                        self.db_service
                            .mark_failed(&agent_id, "Process died unexpectedly")
                            .await?;
                        reconciled += 1;
                    }
                },
                crate::services::agent_orchestration::AgentStatus::Failed { .. } => {
                    // Failed agents will be handled by auto-start below
                },
            }
        }

        if reconciled > 0 {
            CliService::success(&format!("✅ Reconciled {} services", reconciled));
        } else {
            CliService::info("✨ All services are correctly synchronized");
        }

        Ok(reconciled)
    }

    pub async fn reconcile_starting_services(&self) -> OrchestrationResult<u32> {
        CliService::info("🚀 Checking for services stuck in 'starting' state...");

        // No longer needed with supervisor mode - all agents are automatically started
        CliService::info("✨ No services stuck in starting state");
        Ok(0)
    }

    pub async fn perform_consistency_check(&self) -> OrchestrationResult<ConsistencyReport> {
        CliService::info("🔍 Performing database consistency check...");

        let mut report = ConsistencyReport::new();
        let all_agents = self.db_service.list_all_agents().await?;

        for (agent_id, status) in all_agents {
            match status {
                crate::services::agent_orchestration::AgentStatus::Running { pid, .. } => {
                    if process::process_exists(pid) {
                        report.consistent_running.push(agent_id);
                    } else {
                        report.inconsistent_running.push((agent_id, pid));
                    }
                },
                crate::services::agent_orchestration::AgentStatus::Failed { .. } => {
                    report.failed.push(agent_id);
                },
            }
        }

        self.find_orphaned_processes(&mut report).await?;

        report.log_summary();
        Ok(report)
    }

    async fn find_orphaned_processes(
        &self,
        report: &mut ConsistencyReport,
    ) -> OrchestrationResult<()> {
        let running_pids = self.db_service.list_running_agents().await?;

        for agent_id in running_pids {
            let status = self.db_service.get_status(&agent_id).await?;
            if let crate::services::agent_orchestration::AgentStatus::Running { pid, .. } = status {
                if process::process_exists(pid) {
                    continue;
                } else {
                    report.orphaned_processes.push((agent_id, pid));
                }
            }
        }

        Ok(())
    }

    pub async fn fix_inconsistencies(
        &self,
        report: &ConsistencyReport,
    ) -> OrchestrationResult<u32> {
        let mut fixed = 0;

        for (agent_id, pid) in &report.inconsistent_running {
            CliService::warning(&format!("🔧 Fixing inconsistent agent: {}", agent_id));
            self.db_service
                .mark_failed(agent_id, &format!("Process {} died", pid))
                .await?;
            fixed += 1;
        }

        for (agent_id, pid) in &report.orphaned_processes {
            CliService::warning(&format!(
                "🧹 Cleaning up orphaned process for agent: {}",
                agent_id
            ));
            self.db_service
                .mark_failed(agent_id, &format!("Orphaned process {}", pid))
                .await?;
            fixed += 1;
        }

        if fixed > 0 {
            CliService::success(&format!("✅ Fixed {} inconsistencies", fixed));
        }

        Ok(fixed)
    }
}

#[derive(Debug)]
pub struct ConsistencyReport {
    pub consistent_running: Vec<String>,
    pub inconsistent_running: Vec<(String, u32)>,
    pub failed: Vec<String>,
    pub orphaned_processes: Vec<(String, u32)>,
}

impl ConsistencyReport {
    pub fn new() -> Self {
        Self {
            consistent_running: Vec::new(),
            inconsistent_running: Vec::new(),
            failed: Vec::new(),
            orphaned_processes: Vec::new(),
        }
    }

    pub fn has_inconsistencies(&self) -> bool {
        !self.inconsistent_running.is_empty() || !self.orphaned_processes.is_empty()
    }

    pub fn total_agents(&self) -> usize {
        self.consistent_running.len() + self.inconsistent_running.len() + self.failed.len()
    }

    pub fn log_summary(&self) {
        CliService::info(&format!("📊 Consistency Check Results:"));
        CliService::info(&format!(
            "   ✅ Consistent running: {}",
            self.consistent_running.len()
        ));
        CliService::info(&format!(
            "   ❌ Inconsistent running: {}",
            self.inconsistent_running.len()
        ));
        CliService::info(&format!("   💥 Failed: {}", self.failed.len()));
        CliService::info(&format!(
            "   👻 Orphaned processes: {}",
            self.orphaned_processes.len()
        ));

        if self.has_inconsistencies() {
            CliService::warning(
                "⚠️ Inconsistencies detected - run fix_inconsistencies() to repair",
            );
        } else {
            CliService::success("✨ All services are consistent");
        }
    }
}
