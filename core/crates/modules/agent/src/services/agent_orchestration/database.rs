//! Database service layer for agent orchestration operations
//!
//! This service uses:
//! - AgentRegistry (config) for agent definitions (name, port, card)
//! - AgentServiceRepository (database) for runtime state (PID, status, running/stopped)

use crate::repository::AgentServiceRepository;
use crate::services::agent_orchestration::process;
use crate::services::agent_orchestration::{AgentStatus, OrchestrationError, OrchestrationResult};
use crate::services::registry::AgentRegistry;
use systemprompt_models::services::AgentConfig;

#[derive(Debug)]
pub struct AgentDatabaseService {
    pub repository: AgentServiceRepository,
    pub registry: AgentRegistry,
}

impl AgentDatabaseService {
    pub async fn new(repository: AgentServiceRepository) -> OrchestrationResult<Self> {
        let registry = AgentRegistry::new().await.map_err(|e| {
            OrchestrationError::Database(format!("Failed to load agent registry: {}", e))
        })?;

        Ok(Self {
            repository,
            registry,
        })
    }

    pub async fn register_agent(
        &self,
        name: &str,
        pid: u32,
        port: u16,
        auth: &str,
    ) -> OrchestrationResult<String> {
        self.repository
            .register_agent(name, pid, port, auth)
            .await
            .map_err(|e| OrchestrationError::Database(e.to_string()))
    }

    pub async fn get_status(&self, agent_name: &str) -> OrchestrationResult<AgentStatus> {
        let row = self
            .repository
            .get_agent_status(agent_name)
            .await
            .map_err(|e| OrchestrationError::Database(e.to_string()))?;

        match row {
            Some(r) => match (r.pid, r.status.as_str()) {
                (Some(pid), "running") => {
                    let pid = pid as u32;
                    if process::process_exists(pid) {
                        Ok(AgentStatus::Running {
                            pid,
                            port: r.port as u16,
                        })
                    } else {
                        self.mark_failed(agent_name, "Process died unexpectedly")
                            .await?;
                        Ok(AgentStatus::Failed {
                            reason: "Process died unexpectedly".to_string(),
                            last_attempt: None,
                            retry_count: 0,
                        })
                    }
                },
                (_, "failed") | (_, "crashed") => {
                    let error_msg = self
                        .get_error_message(agent_name)
                        .await
                        .unwrap_or_else(|_| "Unknown failure".to_string());
                    Ok(AgentStatus::Failed {
                        reason: error_msg,
                        last_attempt: None,
                        retry_count: 0,
                    })
                },
                _ => {
                    self.mark_failed(agent_name, "Invalid database state")
                        .await?;
                    Ok(AgentStatus::Failed {
                        reason: "Invalid database state".to_string(),
                        last_attempt: None,
                        retry_count: 0,
                    })
                },
            },
            None => Ok(AgentStatus::Failed {
                reason: "No service record found".to_string(),
                last_attempt: None,
                retry_count: 0,
            }),
        }
    }

    pub async fn mark_failed(&self, agent_name: &str, reason: &str) -> OrchestrationResult<()> {
        self.repository
            .mark_error(agent_name, reason)
            .await
            .map_err(|e| OrchestrationError::Database(e.to_string()))?;

        self.repository
            .mark_crashed(agent_name)
            .await
            .map_err(|e| OrchestrationError::Database(e.to_string()))
    }

    pub async fn mark_crashed(&self, agent_name: &str) -> OrchestrationResult<()> {
        self.mark_failed(agent_name, "Process crashed").await
    }

    pub async fn get_error_message(&self, agent_name: &str) -> OrchestrationResult<String> {
        let row = self
            .repository
            .get_agent_status(agent_name)
            .await
            .map_err(|e| OrchestrationError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(format!("Status: {}", r.status)),
            None => Ok("No service record".to_string()),
        }
    }

    pub async fn mark_error(
        &self,
        agent_name: &str,
        error_message: &str,
    ) -> OrchestrationResult<()> {
        self.repository
            .mark_error(agent_name, error_message)
            .await
            .map_err(|e| OrchestrationError::Database(e.to_string()))
    }

    pub async fn list_running_agents(&self) -> OrchestrationResult<Vec<String>> {
        let rows = self
            .repository
            .list_running_agents()
            .await
            .map_err(|e| OrchestrationError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|row| row.name).collect())
    }

    /// List all agents from config with their current runtime status
    ///
    /// Source of truth split:
    /// - Agent definitions (UUID, name, port) come from AgentRegistry (config)
    /// - Runtime state (PID, status) comes from services table (database)
    pub async fn list_all_agents(&self) -> OrchestrationResult<Vec<(String, AgentStatus)>> {
        // Get all agent definitions from config
        let agent_configs = self.registry.list_agents().await.map_err(|e| {
            OrchestrationError::Database(format!("Failed to list agents from config: {}", e))
        })?;

        let mut agents = Vec::new();

        // For each agent in config, check its runtime status
        for agent_config in agent_configs {
            let agent_name = &agent_config.name;

            // Get runtime status from database (if exists)
            let status = self.get_status(agent_name).await?;

            agents.push((agent_name.clone(), status));
        }

        Ok(agents)
    }

    pub async fn agent_exists(&self, agent_name: &str) -> OrchestrationResult<bool> {
        self.registry
            .get_agent(agent_name)
            .await
            .map(|_| true)
            .or_else(|_| Ok(false))
    }

    /// Get agent configuration from registry (config file)
    pub async fn get_agent_config(&self, agent_name: &str) -> OrchestrationResult<AgentConfig> {
        let agent_config = self.registry.get_agent(agent_name).await.map_err(|e| {
            OrchestrationError::AgentNotFound(format!(
                "Agent {} not found in config: {}",
                agent_name, e
            ))
        })?;

        Ok(agent_config)
    }

    pub async fn cleanup_orphaned_services(&self) -> OrchestrationResult<u64> {
        let rows = self
            .repository
            .list_running_agent_pids()
            .await
            .map_err(|e| OrchestrationError::Database(e.to_string()))?;

        let mut cleaned = 0u64;

        for row in rows {
            let pid = row.pid as u32;
            if !process::process_exists(pid) {
                self.mark_crashed(&row.name).await?;
                cleaned += 1;
            }
        }

        Ok(cleaned)
    }

    pub async fn remove_agent_service(&self, agent_name: &str) -> OrchestrationResult<()> {
        self.repository
            .remove_agent_service(agent_name)
            .await
            .map_err(|e| OrchestrationError::Database(e.to_string()))
    }

    pub async fn update_health_status(
        &self,
        agent_name: &str,
        health_status: &str,
    ) -> OrchestrationResult<()> {
        self.repository
            .update_health_status(agent_name, health_status)
            .await
            .map_err(|e| OrchestrationError::Database(e.to_string()))
    }

    pub async fn get_unresponsive_agents(
        &self,
        _max_failures: u32,
    ) -> OrchestrationResult<Vec<(String, Option<u32>)>> {
        use crate::services::agent_orchestration::monitor::check_a2a_agent_health;

        let agents = self.list_all_agents().await?;

        let mut unresponsive = Vec::new();
        for (agent_name, status) in agents {
            if let AgentStatus::Running { pid, port, .. } = status {
                let is_healthy = check_a2a_agent_health(port, 10).await.unwrap_or(false);

                if !is_healthy {
                    unresponsive.push((agent_name, Some(pid)));
                }
            }
        }

        Ok(unresponsive)
    }
}
