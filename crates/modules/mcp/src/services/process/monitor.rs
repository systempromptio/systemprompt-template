// NOTE: McpOrchestrator was removed - this code is unused/deprecated
// use crate::services::orchestrator::McpOrchestrator;
use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, Instant};
use systemprompt_core_logging::CliService;
use systemprompt_models::repository::ServiceRepository;

const HEALTH_CHECK_TIMEOUT_SECS: u64 = 5;
const CONSECUTIVE_FAILURES_THRESHOLD: usize = 3;

#[derive(Debug, Clone)]
struct HealthCheckFailure {
    count: usize,
}

pub async fn is_service_healthy(port: u16) -> Result<bool> {
    is_port_responsive(port).await
}

async fn is_port_responsive(port: u16) -> Result<bool> {
    use tokio::net::TcpStream;
    use tokio::time::timeout;

    match timeout(
        Duration::from_secs(HEALTH_CHECK_TIMEOUT_SECS),
        TcpStream::connect(format!("127.0.0.1:{port}")),
    )
    .await
    {
        Ok(Ok(_)) => Ok(true),
        _ => Ok(false),
    }
}

pub async fn is_process_running(pid: u32) -> Result<bool> {
    Ok(std::path::Path::new(&format!("/proc/{pid}")).exists())
}

async fn is_service_fully_healthy(port: u16, pid: Option<i32>) -> Result<bool> {
    let port_responsive = is_port_responsive(port).await.unwrap_or(false);

    let process_alive = if let Some(pid) = pid {
        is_process_running(pid as u32).await.unwrap_or(false)
    } else {
        false
    };

    Ok(port_responsive && process_alive)
}

pub async fn get_process_info(pid: u32) -> Result<Option<ProcessInfo>> {
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "pid,ppid,cmd"])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.len() < 2 {
        return Ok(None);
    }

    let parts: Vec<&str> = lines[1].split_whitespace().collect();
    if parts.len() < 3 {
        return Ok(None);
    }

    let pid: u32 = parts[0]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid PID: {}", parts[0]))?;
    let parent_pid: u32 = parts[1]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid PPID: {}", parts[1]))?;

    if pid == 0 {
        return Err(anyhow::anyhow!("PID cannot be 0"));
    }

    Ok(Some(ProcessInfo {
        pid,
        ppid: parent_pid,
        command: parts[2..].join(" "),
    }))
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub command: String,
}

// Deprecated - McpOrchestrator was removed
#[allow(dead_code)]
pub async fn monitor_processes(
    db_pool: systemprompt_core_database::DbPool,
    _orchestrator_placeholder: (), // Arc<McpOrchestrator> was removed
) -> Result<()> {
    let repository = ServiceRepository::new(db_pool.clone());
    let mut restart_backoff: HashMap<String, (usize, Instant)> = HashMap::new();
    let mut health_failures: HashMap<String, HealthCheckFailure> = HashMap::new();

    loop {
        let services = repository.get_mcp_services().await?;

        for service in services {
            if service.status == "running" {
                let port = service.port as u16;
                let pid = service.pid;

                let is_healthy = is_service_fully_healthy(port, pid).await.unwrap_or(false);

                if is_healthy {
                    health_failures.remove(&service.name);
                } else {
                    let failure = health_failures
                        .entry(service.name.clone())
                        .or_insert_with(|| HealthCheckFailure { count: 0 });

                    failure.count += 1;

                    if failure.count == 1 {
                        CliService::warning(&format!(
                            "⚠️  Health check failed for {}: port {} (PID {:?}), will monitor \
                             closely",
                            service.name, port, pid
                        ));
                    }

                    if failure.count >= CONSECUTIVE_FAILURES_THRESHOLD {
                        let port_healthy = is_port_responsive(port).await.unwrap_or(false);
                        let process_healthy = if let Some(p) = pid {
                            is_process_running(p as u32).await.unwrap_or(false)
                        } else {
                            false
                        };

                        let failure_reason = match (pid.is_some(), port_healthy, process_healthy) {
                            (true, false, true) => "port not responding",
                            (_, _, false) => "process not found",
                            _ => "unknown",
                        };

                        repository.mark_service_crashed(&service.name).await?;

                        CliService::warning(&format!(
                            "🔴 Marked MCP service as crashed: {} ({}, {} consecutive failures)",
                            service.name, failure_reason, failure.count
                        ));

                        let restart_info = restart_backoff
                            .entry(service.name.clone())
                            .or_insert((0, Instant::now()));

                        let (attempts, last_restart) = *restart_info;
                        let backoff = Duration::from_secs(2u64.pow(attempts.min(5) as u32));

                        if last_restart.elapsed() > backoff && attempts < 5 {
                            CliService::warning(&format!(
                                "🔄 Auto-restart needed for crashed service: {} (attempt {}) - \
                                 restart functionality removed with McpOrchestrator",
                                service.name,
                                attempts + 1
                            ));
                            // orchestrator.start_services was removed with
                            // McpOrchestrator
                            // This function is deprecated and should not be
                            // used
                        } else if attempts >= 5 {
                            CliService::error(&format!(
                                "🛑 Service {} exceeded restart limit (5 attempts)",
                                service.name
                            ));
                        }
                    }
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
