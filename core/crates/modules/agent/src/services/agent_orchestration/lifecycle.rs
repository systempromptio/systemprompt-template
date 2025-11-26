//! Lifecycle Management - Agent Start/Stop/Restart Operations

use std::time::Duration;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::{CliService, LogService};

use crate::services::agent_orchestration::database::AgentDatabaseService;
use crate::services::agent_orchestration::{
    process, AgentStatus, OrchestrationError, OrchestrationResult,
};

#[derive(Debug)]
pub struct AgentLifecycle {
    db_service: AgentDatabaseService,
    logger: LogService,
}

impl AgentLifecycle {
    pub async fn new(db_pool: DbPool) -> anyhow::Result<Self> {
        use crate::repository::AgentServiceRepository;

        let logger = LogService::system(db_pool.clone());
        let agent_service_repo = AgentServiceRepository::new(db_pool.clone());
        let db_service = AgentDatabaseService::new(agent_service_repo).await?;

        Ok(Self { db_service, logger })
    }

    pub async fn start_agent(&self, agent_name: &str) -> OrchestrationResult<String> {
        CliService::info(&format!("🚀 Starting agent: {}", agent_name));

        let result = async {
            let current_status = self.db_service.get_status(agent_name).await?;
            match current_status {
                AgentStatus::Running { .. } => {
                    return Err(OrchestrationError::AgentAlreadyRunning(
                        agent_name.to_string(),
                    ));
                },
                AgentStatus::Failed { .. } => {
                    CliService::info("   Agent previously failed, attempting restart...");
                },
            }

            let agent_config = self.db_service.get_agent_config(agent_name).await?;
            self.validate_prerequisites(agent_config.port).await?;

            let pid = self
                .spawn_detached_process(agent_name, agent_config.port)
                .await?;

            let auth_type = self.determine_auth_type(&agent_config).await?;

            let service_id = self
                .db_service
                .register_agent(&agent_config.name, pid, agent_config.port, &auth_type)
                .await?;

            self.verify_startup(agent_name, agent_config.port).await?;

            self.logger
                .log(
                    systemprompt_core_logging::LogLevel::Info,
                    "agent_lifecycle",
                    &format!("Agent started: {}", agent_config.name),
                    Some(serde_json::json!({
                        "agent_name": agent_name,
                        "port": agent_config.port,
                        "pid": pid,
                        "service_id": service_id,
                    })),
                )
                .await
                .ok();

            CliService::success(&format!("✅ Agent {} started successfully", agent_name));
            Ok(service_id)
        }
        .await;

        if let Err(ref e) = result {
            self.logger
                .error(
                    "agent_lifecycle",
                    &format!("Failed to start agent {}: {}", agent_name, e),
                )
                .await
                .ok();
        }

        result
    }

    pub async fn disable_agent(&self, agent_name: &str) -> OrchestrationResult<()> {
        CliService::info(&format!("🛑 Disabling agent: {}", agent_name));

        let status = self.db_service.get_status(agent_name).await?;

        if let AgentStatus::Running { pid, .. } = status {
            if process::kill_process(pid) {
                CliService::success(&format!("   Killed process {}", pid));
            } else {
                CliService::warning(&format!("   Failed to kill process {}", pid));
            }
        }

        // Agent enabled state is managed in config files, not database
        self.db_service.remove_agent_service(agent_name).await?;

        CliService::success(&format!("✅ Agent {} disabled", agent_name));
        Ok(())
    }

    pub async fn enable_agent(&self, agent_name: &str) -> OrchestrationResult<String> {
        CliService::info(&format!("🚀 Enabling agent: {}", agent_name));

        // Agent enabled state is managed in config files, not database
        self.start_agent(agent_name).await
    }

    pub async fn restart_agent(&self, agent_name: &str) -> OrchestrationResult<String> {
        CliService::info(&format!("🔄 Restarting agent: {}", agent_name));

        let status = self.db_service.get_status(agent_name).await?;
        if let AgentStatus::Running { pid, .. } = status {
            if process::kill_process(pid) {
                CliService::success(&format!("   Killed process {}", pid));
            } else {
                CliService::warning(&format!("   Failed to kill process {}", pid));
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        self.start_agent(agent_name).await
    }

    pub async fn cleanup_crashed_agent(&self, agent_name: &str) -> OrchestrationResult<()> {
        let status = self.db_service.get_status(agent_name).await?;

        if let AgentStatus::Running { pid, .. } = status {
            if !process::process_exists(pid) {
                self.db_service.mark_crashed(agent_name).await?;
                CliService::info(&format!(
                    "🧹 Marked crashed agent {} as failed in database",
                    agent_name
                ));
            }
        }

        Ok(())
    }

    async fn validate_prerequisites(&self, port: u16) -> OrchestrationResult<()> {
        use super::port_manager::PortManager;

        let port_manager = PortManager::new();

        if process::is_port_in_use(port) {
            match port_manager.cleanup_port_if_needed(port).await {
                Ok(_) => {
                    self.logger
                        .info("agent_lifecycle", &format!("Cleaned up port {}", port))
                        .await
                        .ok();
                },
                Err(e) => {
                    let error_msg = format!("Port {} is in use and cleanup failed: {}", port, e);
                    self.logger.error("agent_lifecycle", &error_msg).await.ok();
                    return Err(e);
                },
            }
        }

        Ok(())
    }

    async fn spawn_detached_process(
        &self,
        agent_name: &str,
        port: u16,
    ) -> OrchestrationResult<u32> {
        process::spawn_detached_process(agent_name, port).await
    }

    async fn verify_startup(&self, agent_name: &str, port: u16) -> OrchestrationResult<()> {
        for attempt in 1..=15 {
            tokio::time::sleep(Duration::from_millis(1000 * attempt)).await;

            match self.check_port_responsiveness(port, 5).await {
                Ok(true) => {
                    return Ok(());
                },
                Ok(false) | Err(_) => {
                    continue;
                },
            }
        }

        let error_msg = "Agent failed to become responsive within timeout";
        self.db_service.mark_error(agent_name, error_msg).await?;
        Err(OrchestrationError::HealthCheckTimeout(
            agent_name.to_string(),
        ))
    }

    async fn check_port_responsiveness(
        &self,
        port: u16,
        timeout_secs: u64,
    ) -> OrchestrationResult<bool> {
        use std::time::Duration;
        use tokio::net::TcpStream;
        use tokio::time::timeout;

        let address = format!("127.0.0.1:{}", port);
        match timeout(
            Duration::from_secs(timeout_secs),
            TcpStream::connect(&address),
        )
        .await
        {
            Ok(Ok(_)) => Ok(true),
            Ok(Err(_)) | Err(_) => Ok(false),
        }
    }

    async fn determine_auth_type(
        &self,
        agent_config: &systemprompt_models::services::AgentConfig,
    ) -> OrchestrationResult<String> {
        if agent_config.oauth.required {
            Ok(agent_config
                .oauth
                .scopes
                .first()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "user".to_string()))
        } else {
            Ok("anonymous".to_string())
        }
    }
}

pub async fn start_agent(pool: &DbPool, agent_name: &str) -> OrchestrationResult<String> {
    let lifecycle = AgentLifecycle::new(pool.clone())
        .await
        .map_err(|e| OrchestrationError::Generic(e))?;
    lifecycle.start_agent(agent_name).await
}

pub async fn enable_agent(pool: &DbPool, agent_name: &str) -> OrchestrationResult<String> {
    let lifecycle = AgentLifecycle::new(pool.clone())
        .await
        .map_err(|e| OrchestrationError::Generic(e))?;
    lifecycle.enable_agent(agent_name).await
}

pub async fn disable_agent(pool: &DbPool, agent_name: &str) -> OrchestrationResult<()> {
    let lifecycle = AgentLifecycle::new(pool.clone())
        .await
        .map_err(|e| OrchestrationError::Generic(e))?;
    lifecycle.disable_agent(agent_name).await
}

pub async fn restart_agent(pool: &DbPool, agent_name: &str) -> OrchestrationResult<String> {
    let lifecycle = AgentLifecycle::new(pool.clone())
        .await
        .map_err(|e| OrchestrationError::Generic(e))?;
    lifecycle.restart_agent(agent_name).await
}
