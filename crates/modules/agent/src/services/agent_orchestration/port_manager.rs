//! Port Management - Handle Port Conflicts and Orphaned Processes

use anyhow::{Context, Result};
use std::process::Command;
use std::time::Duration;
use systemprompt_core_logging::CliService;

use crate::services::agent_orchestration::{process, OrchestrationError, OrchestrationResult};

#[derive(Debug, Copy, Clone)]
pub struct PortManager;

impl PortManager {
    pub fn new() -> Self {
        Self
    }

    pub fn find_process_using_port(&self, port: u16) -> Result<Option<u32>> {
        let output = Command::new("lsof")
            .arg("-ti")
            .arg(format!(":{}", port))
            .output()
            .context("Failed to run lsof command")?;

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let pid_str = stdout.trim();

        if pid_str.is_empty() {
            return Ok(None);
        }

        let pid = pid_str
            .parse::<u32>()
            .context("Failed to parse PID from lsof output")?;

        Ok(Some(pid))
    }

    pub fn get_process_info(&self, pid: u32) -> Result<Option<ProcessInfo>> {
        let output = Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .arg("-o")
            .arg("pid,comm,args")
            .arg("--no-headers")
            .output()
            .context("Failed to run ps command")?;

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.trim();

        if line.is_empty() {
            return Ok(None);
        }

        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 3 {
            return Ok(None);
        }

        let command_line = parts[2].trim();

        Ok(Some(ProcessInfo {
            pid,
            command: command_line.to_string(),
        }))
    }

    pub fn is_agent_process(&self, pid: u32) -> bool {
        if let Ok(Some(info)) = self.get_process_info(pid) {
            info.command.contains("systemprompt")
                && (info.command.contains("agents run") || info.command.contains("agent-worker"))
        } else {
            false
        }
    }

    pub async fn kill_process_on_port(&self, port: u16) -> OrchestrationResult<bool> {
        let pid = match self.find_process_using_port(port) {
            Ok(Some(p)) => p,
            Ok(None) => {
                return Ok(false);
            },
            Err(e) => {
                return Err(OrchestrationError::ProcessSpawnFailed(format!(
                    "Failed to check port {}: {}",
                    port, e
                )));
            },
        };

        if !self.is_agent_process(pid) {
            return Err(OrchestrationError::ProcessSpawnFailed(format!(
                "Port {} is in use by non-agent process (PID {}). Please free the port manually.",
                port, pid
            )));
        }

        CliService::warning(&format!(
            "🧹 Killing orphaned agent process PID {} on port {}",
            pid, port
        ));

        if !process::kill_process(pid) {
            return Err(OrchestrationError::ProcessSpawnFailed(format!(
                "Failed to kill process {} on port {}",
                pid, port
            )));
        }

        self.wait_for_port_available(port, 5).await?;

        CliService::success(&format!("✅ Port {} is now available", port));
        Ok(true)
    }

    pub async fn wait_for_port_available(
        &self,
        port: u16,
        timeout_secs: u64,
    ) -> OrchestrationResult<()> {
        let check_interval = Duration::from_millis(100);
        let max_checks = (timeout_secs * 1000) / 100;

        for _ in 0..max_checks {
            if !process::is_port_in_use(port) {
                return Ok(());
            }
            tokio::time::sleep(check_interval).await;
        }

        Err(OrchestrationError::ProcessSpawnFailed(format!(
            "Port {} did not become available within {} seconds",
            port, timeout_secs
        )))
    }

    pub async fn cleanup_port_if_needed(&self, port: u16) -> OrchestrationResult<()> {
        if !process::is_port_in_use(port) {
            return Ok(());
        }

        match self.find_process_using_port(port) {
            Ok(Some(pid)) => {
                if self.is_agent_process(pid) {
                    CliService::warning(&format!(
                        "⚠️  Port {} occupied by orphaned agent process (PID {})",
                        port, pid
                    ));
                    self.kill_process_on_port(port).await?;
                } else {
                    let info = self
                        .get_process_info(pid)
                        .ok()
                        .flatten()
                        .map(|i| i.command)
                        .unwrap_or_else(|| "unknown".to_string());

                    return Err(OrchestrationError::ProcessSpawnFailed(format!(
                        "Port {} is in use by non-agent process (PID {}): {}\n\
                        Please stop the process manually or choose a different port.",
                        port, pid, info
                    )));
                }
            },
            Ok(None) => {
                return Err(OrchestrationError::ProcessSpawnFailed(format!(
                    "Port {} appears to be in use but process cannot be identified",
                    port
                )));
            },
            Err(e) => {
                return Err(OrchestrationError::ProcessSpawnFailed(format!(
                    "Failed to check port {}: {}",
                    port, e
                )));
            },
        }

        Ok(())
    }

    pub async fn cleanup_agent_ports(&self, ports: &[u16]) -> OrchestrationResult<u32> {
        let mut cleaned = 0;

        for &port in ports {
            if process::is_port_in_use(port) {
                match self.cleanup_port_if_needed(port).await {
                    Ok(_) => cleaned += 1,
                    Err(e) => {
                        CliService::error(&format!("❌ Failed to cleanup port {}: {}", port, e));
                        return Err(e);
                    },
                }
            }
        }

        if cleaned > 0 {
            CliService::success(&format!("✅ Cleaned up {} ports", cleaned));
        }

        Ok(cleaned)
    }

    pub async fn verify_all_ports_available(&self, ports: &[u16]) -> OrchestrationResult<()> {
        let mut blocked_ports = Vec::new();

        for &port in ports {
            if process::is_port_in_use(port) {
                if let Ok(Some(pid)) = self.find_process_using_port(port) {
                    blocked_ports.push((port, pid));
                }
            }
        }

        if !blocked_ports.is_empty() {
            let port_info: Vec<String> = blocked_ports
                .iter()
                .map(|(port, pid)| {
                    let info = self
                        .get_process_info(*pid)
                        .ok()
                        .flatten()
                        .map(|i| i.command)
                        .unwrap_or_else(|| "unknown".to_string());
                    format!("  • Port {} - PID {} ({})", port, pid, info)
                })
                .collect();

            return Err(OrchestrationError::ProcessSpawnFailed(format!(
                "The following ports are still in use:\n{}",
                port_info.join("\n")
            )));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub command: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_manager_creation() {
        let manager = PortManager::new();
        assert!(true);
    }
}
