//! Health Monitoring and Process Validation

use anyhow::Result;
use std::time::Duration;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::CliService;
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::services::agent_orchestration::database::AgentDatabaseService;
use crate::services::agent_orchestration::{process, OrchestrationResult};

#[derive(Debug)]
pub struct AgentMonitor {
    db_service: AgentDatabaseService,
}

impl AgentMonitor {
    pub async fn new(db_pool: DbPool) -> OrchestrationResult<Self> {
        use crate::repository::AgentServiceRepository;

        let agent_service_repo = AgentServiceRepository::new(db_pool);
        let db_service = AgentDatabaseService::new(agent_service_repo).await?;

        Ok(Self { db_service })
    }

    pub async fn comprehensive_health_check(
        &self,
        agent_id: &str,
    ) -> OrchestrationResult<HealthCheckResult> {
        let status = self.db_service.get_status(agent_id).await?;

        match status {
            crate::services::agent_orchestration::AgentStatus::Running { pid, port } => {
                if !process::process_exists(pid) {
                    return Ok(HealthCheckResult {
                        healthy: false,
                        message: format!("Process {} no longer exists", pid),
                        response_time_ms: 0,
                    });
                }

                match perform_tcp_health_check("127.0.0.1", port).await {
                    Ok(result) => Ok(result),
                    Err(e) => Ok(HealthCheckResult {
                        healthy: false,
                        message: format!("TCP check failed: {}", e),
                        response_time_ms: 0,
                    }),
                }
            },
            _ => Ok(HealthCheckResult {
                healthy: false,
                message: format!("Agent {} not in running state", agent_id),
                response_time_ms: 0,
            }),
        }
    }

    pub async fn monitor_all_agents(&self) -> OrchestrationResult<MonitoringReport> {
        let agents = self.db_service.list_all_agents().await?;
        let mut report = MonitoringReport::new();

        for (agent_id, status) in agents {
            match status {
                crate::services::agent_orchestration::AgentStatus::Running { pid, port } => {
                    if process::process_exists(pid) {
                        let health_result = perform_tcp_health_check("127.0.0.1", port).await?;
                        if health_result.healthy {
                            report.healthy_agents.push(agent_id);
                        } else {
                            report.unhealthy_agents.push(agent_id);
                        }
                    } else {
                        self.db_service
                            .mark_failed(&agent_id, "Process died")
                            .await?;
                        report.failed_agents.push(agent_id);
                    }
                },
                crate::services::agent_orchestration::AgentStatus::Failed { .. } => {
                    report.failed_agents.push(agent_id);
                },
            }
        }

        Ok(report)
    }

    pub async fn cleanup_unresponsive_agents(&self, max_failures: u32) -> OrchestrationResult<u32> {
        CliService::info("🧹 Cleaning up unresponsive agents...");

        let unresponsive_agents = self
            .db_service
            .get_unresponsive_agents(max_failures)
            .await?;
        let mut cleaned_up = 0;

        for (agent_id, pid_opt) in unresponsive_agents {
            if let Some(pid) = pid_opt {
                CliService::warning(&format!(
                    "🔴 Killing unresponsive agent {} (PID {})",
                    agent_id, pid
                ));

                if process::kill_process(pid) {
                    self.db_service.mark_crashed(&agent_id).await?;
                    cleaned_up += 1;
                    CliService::success(&format!("✅ Cleaned up agent {}", agent_id));
                } else {
                    CliService::error(&format!(
                        "❌ Failed to kill agent {} (PID {})",
                        agent_id, pid
                    ));
                }
            }
        }

        if cleaned_up > 0 {
            CliService::info(&format!("🧹 Cleaned up {} unresponsive agents", cleaned_up));
        } else {
            CliService::info("✨ No unresponsive agents found");
        }

        Ok(cleaned_up)
    }
}

#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub healthy: bool,
    pub message: String,
    pub response_time_ms: u64,
}

#[derive(Debug)]
pub struct MonitoringReport {
    pub healthy_agents: Vec<String>,
    pub unhealthy_agents: Vec<String>,
    pub failed_agents: Vec<String>,
}

impl MonitoringReport {
    pub fn new() -> Self {
        Self {
            healthy_agents: Vec::new(),
            unhealthy_agents: Vec::new(),
            failed_agents: Vec::new(),
        }
    }

    pub fn total_agents(&self) -> usize {
        self.healthy_agents.len() + self.unhealthy_agents.len() + self.failed_agents.len()
    }

    pub fn healthy_percentage(&self) -> f64 {
        let total = self.total_agents();
        if total == 0 {
            0.0
        } else {
            (self.healthy_agents.len() as f64 / total as f64) * 100.0
        }
    }
}

pub async fn check_agent_health(agent_id: &str) -> Result<HealthCheckResult> {
    let port = get_agent_port_simple(agent_id).await?;
    perform_tcp_health_check("127.0.0.1", port).await
}

async fn perform_tcp_health_check(host: &str, port: u16) -> Result<HealthCheckResult> {
    let start = std::time::Instant::now();
    let address = format!("{}:{}", host, port);

    CliService::info(&format!(
        "🏥 Health check: attempting TCP connection to {}",
        address
    ));

    match timeout(Duration::from_secs(15), TcpStream::connect(&address)).await {
        Ok(Ok(_)) => {
            let response_time = start.elapsed().as_millis() as u64;
            CliService::info(&format!(
                "✅ Health check PASSED: {} ({}ms)",
                address, response_time
            ));
            Ok(HealthCheckResult {
                healthy: true,
                message: "TCP connection successful".to_string(),
                response_time_ms: response_time,
            })
        },
        Ok(Err(e)) => {
            CliService::warning(&format!(
                "❌ Health check FAILED: {} - Connection error: {}",
                address, e
            ));
            Ok(HealthCheckResult {
                healthy: false,
                message: format!("Connection failed: {}", e),
                response_time_ms: 0,
            })
        },
        Err(_) => {
            CliService::error(&format!(
                "⏱️  Health check TIMEOUT: {} - no response in 5 seconds",
                address
            ));
            Ok(HealthCheckResult {
                healthy: false,
                message: "Connection timeout".to_string(),
                response_time_ms: 5000,
            })
        },
    }
}

async fn get_agent_port_simple(agent_id: &str) -> Result<u16> {
    let port_str = agent_id
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>();

    if port_str.is_empty() {
        return Ok(8000);
    }

    let port_num: u16 = port_str.parse().unwrap_or(8000);
    Ok(8000 + (port_num % 1000))
}

pub async fn check_agent_responsiveness(agent_id: &str, timeout_secs: u64) -> Result<bool> {
    let port = get_agent_port_simple(agent_id).await?;
    let address = format!("127.0.0.1:{}", port);

    match timeout(
        Duration::from_secs(timeout_secs),
        TcpStream::connect(&address),
    )
    .await
    {
        Ok(Ok(_)) => {
            CliService::success(&format!("✅ Agent {} is responsive", agent_id));
            Ok(true)
        },
        Ok(Err(e)) => {
            CliService::warning(&format!("⚠️ Agent {} connection failed: {}", agent_id, e));
            Ok(false)
        },
        Err(_) => {
            CliService::warning(&format!(
                "⚠️ Agent {} connection timeout after {}s",
                agent_id, timeout_secs
            ));
            Ok(false)
        },
    }
}

/// Check agent health using A2A protocol /.well-known/agent-card.json endpoint
/// This is the A2A specification-compliant way to verify agent responsiveness
pub async fn check_a2a_agent_health(port: u16, timeout_secs: u64) -> Result<bool> {
    let url = format!("http://localhost:{}/.well-known/agent-card.json", port);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .timeout(Duration::from_secs(timeout_secs))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            // Verify it's actually a valid agent card by checking for basic JSON structure
            match resp.json::<serde_json::Value>().await {
                Ok(json) => {
                    // Basic validation - agent card should have name and url fields
                    let is_valid_card = json.get("name").is_some() && json.get("url").is_some();
                    Ok(is_valid_card)
                },
                Err(_) => Ok(false), // Invalid JSON response
            }
        },
        Ok(_) => Ok(false),  // Non-success HTTP status
        Err(_) => Ok(false), // Connection failed or timeout
    }
}
