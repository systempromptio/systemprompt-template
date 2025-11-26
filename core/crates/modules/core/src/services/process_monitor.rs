use anyhow::Result;
use std::time::Duration;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::CliService;
use systemprompt_models::repository::ServiceRepository;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct ProcessMonitor {
    db_pool: DbPool,
    monitor_handle: Option<JoinHandle<()>>,
    check_interval: Duration,
}

impl ProcessMonitor {
    pub const fn new(db_pool: DbPool) -> Self {
        Self {
            db_pool,
            monitor_handle: None,
            check_interval: Duration::from_secs(30),
        }
    }

    pub const fn with_interval(db_pool: DbPool, interval: Duration) -> Self {
        Self {
            db_pool,
            monitor_handle: None,
            check_interval: interval,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        if self.monitor_handle.is_some() {
            CliService::warning("⚠️ Process monitor already started");
            return Ok(());
        }

        CliService::info("🔍 Starting centralized process monitoring...");

        let db_pool_clone = self.db_pool.clone();
        let interval = self.check_interval;

        let handle = tokio::spawn(async move { Self::monitor_loop(db_pool_clone, interval).await });

        self.monitor_handle = Some(handle);
        CliService::success("✅ Centralized process monitoring started");

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.monitor_handle.take() {
            CliService::info("🛑 Stopping process monitoring...");
            handle.abort();
            CliService::success("✅ Process monitoring stopped");
        }

        Ok(())
    }

    pub const fn is_running(&self) -> bool {
        self.monitor_handle.is_some()
    }

    async fn monitor_loop(db_pool: DbPool, check_interval: Duration) {
        CliService::info(&format!(
            "🔍 Process monitor loop started (checking every {}s)",
            check_interval.as_secs()
        ));

        let mut interval = tokio::time::interval(check_interval);

        loop {
            interval.tick().await;

            if let Err(e) = Self::perform_monitoring_cycle(&db_pool).await {
                CliService::warning(&format!("Monitoring cycle failed: {e}"));
            }
        }
    }

    async fn perform_monitoring_cycle(db_pool: &DbPool) -> Result<()> {
        let repository = ServiceRepository::new(db_pool.clone());
        let services = repository.get_running_services_with_pid().await?;

        if services.is_empty() {
            return Ok(());
        }

        let mut healthy_count = 0;
        let mut crashed_count = 0;

        for service in services {
            if let Some(pid) = service.pid {
                let pid = pid as u32;

                if Self::process_exists(pid) {
                    healthy_count += 1;
                } else {
                    repository.mark_service_crashed(&service.name).await?;

                    crashed_count += 1;
                    CliService::warning(&format!(
                        "🔴 Detected crashed {} service: {} (PID {})",
                        service.module_name, service.name, pid
                    ));
                }
            }
        }

        if crashed_count == 0 {
            CliService::success(&format!("✅ All {healthy_count} services healthy"));
        } else {
            CliService::warning(&format!(
                "⚠️ Health: {healthy_count} healthy, {crashed_count} crashed and updated"
            ));
        }

        Ok(())
    }

    fn process_exists(pid: u32) -> bool {
        std::path::Path::new(&format!("/proc/{pid}")).exists()
    }

    pub async fn health_check_all(&self) -> Result<HealthSummary> {
        CliService::info("🔍 Running health check on all services...");

        let repository = ServiceRepository::new(self.db_pool.clone());
        let services = repository.get_running_services_with_pid().await?;

        let mut summary = HealthSummary::default();

        for service in services {
            if let Some(pid) = service.pid {
                let pid = pid as u32;
                let healthy = Self::process_exists(pid);

                let status_icon = if healthy { "✅" } else { "❌" };
                CliService::info(&format!(
                    "   {} {}/{}: {} (PID {})",
                    status_icon,
                    service.module_name,
                    service.name,
                    if healthy { "healthy" } else { "not running" },
                    pid
                ));

                *summary
                    .modules
                    .entry(service.module_name)
                    .or_insert(ModuleHealth::default()) += if healthy {
                    ModuleHealth {
                        healthy: 1,
                        crashed: 0,
                    }
                } else {
                    ModuleHealth {
                        healthy: 0,
                        crashed: 1,
                    }
                };
            }
        }

        let total_healthy = summary.modules.values().map(|m| m.healthy).sum::<u32>();
        let total_crashed = summary.modules.values().map(|m| m.crashed).sum::<u32>();

        if total_crashed == 0 {
            CliService::success(&format!("✅ All {total_healthy} services are healthy"));
        } else {
            CliService::warning(&format!(
                "⚠️ {}/{} services are healthy",
                total_healthy,
                total_healthy + total_crashed
            ));
        }

        Ok(summary)
    }
}

impl Drop for ProcessMonitor {
    fn drop(&mut self) {
        if let Some(handle) = self.monitor_handle.take() {
            handle.abort();
        }
    }
}

#[derive(Debug, Default)]
pub struct HealthSummary {
    pub modules: std::collections::HashMap<String, ModuleHealth>,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ModuleHealth {
    pub healthy: u32,
    pub crashed: u32,
}

impl std::ops::AddAssign for ModuleHealth {
    fn add_assign(&mut self, other: Self) {
        self.healthy += other.healthy;
        self.crashed += other.crashed;
    }
}

impl HealthSummary {
    pub fn total_healthy(&self) -> u32 {
        self.modules.values().map(|m| m.healthy).sum()
    }

    pub fn total_crashed(&self) -> u32 {
        self.modules.values().map(|m| m.crashed).sum()
    }

    pub fn is_all_healthy(&self) -> bool {
        self.total_crashed() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_exists() {
        let current_pid = std::process::id();
        assert!(ProcessMonitor::process_exists(current_pid));
        assert!(!ProcessMonitor::process_exists(99999999));
    }

    #[test]
    fn test_health_summary() {
        let mut summary = HealthSummary::default();
        *summary.modules.entry("mcp".to_string()).or_default() += ModuleHealth {
            healthy: 2,
            crashed: 1,
        };
        *summary.modules.entry("agent".to_string()).or_default() += ModuleHealth {
            healthy: 1,
            crashed: 0,
        };

        assert_eq!(summary.total_healthy(), 3);
        assert_eq!(summary.total_crashed(), 1);
        assert!(!summary.is_all_healthy());
    }
}
