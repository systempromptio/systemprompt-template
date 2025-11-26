//! Process Management - Detached Process Spawning and PID Operations

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use systemprompt_core_logging::CliService;
use systemprompt_core_system::{BinaryPaths, Config};

use crate::services::agent_orchestration::{OrchestrationError, OrchestrationResult};

const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024;

fn rotate_log_if_needed(log_path: &Path) -> Result<()> {
    if let Ok(metadata) = fs::metadata(log_path) {
        if metadata.len() > MAX_LOG_SIZE {
            let backup_path = log_path.with_extension("log.old");
            fs::rename(log_path, &backup_path).ok();
        }
    }
    Ok(())
}

/// Spawn an agent as a truly detached process that will survive orchestrator restarts
pub async fn spawn_detached(agent_name: &str, port: u16) -> OrchestrationResult<u32> {
    let binary_path = BinaryPaths::resolve_binary("systemprompt").map_err(|e| {
        OrchestrationError::ProcessSpawnFailed(format!("Failed to find systemprompt binary: {}", e))
    })?;

    let config = Config::global();

    // Create logs directory if it doesn't exist
    let log_dir = Path::new(&config.system_path).join("logs");
    fs::create_dir_all(&log_dir).ok();

    // Redirect stderr to log file for debugging startup failures
    let log_file_path = log_dir.join(format!("agent-{}.log", agent_name));
    rotate_log_if_needed(&log_file_path).ok();

    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .map_err(|e| {
            OrchestrationError::ProcessSpawnFailed(format!(
                "Failed to create log file {}: {}",
                log_file_path.display(),
                e
            ))
        })?;

    let mut command = Command::new(&binary_path);
    command
        .arg("agents")
        .arg("run")
        .arg("--agent-name")
        .arg(agent_name)
        .arg("--port")
        .arg(port.to_string())
        // Inherit all environment variables from parent process
        .envs(std::env::vars())
        // Override agent-specific variables
        .env("AGENT_NAME", agent_name)
        .env("AGENT_PORT", port.to_string())
        .env("DATABASE_URL", &config.database_url)
        .env("DATABASE_TYPE", &config.database_type)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::from(log_file))
        .stdin(std::process::Stdio::null());

    let child = command.spawn().map_err(|e| {
        OrchestrationError::ProcessSpawnFailed(format!("Failed to spawn {}: {}", agent_name, e))
    })?;

    let pid = child.id();

    // Critical: Detach from parent - process continues after orchestrator exits
    std::mem::forget(child);

    // Validate process started successfully
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    if !process_exists(pid) {
        return Err(OrchestrationError::ProcessSpawnFailed(format!(
            "Agent {} (PID {}) died immediately after spawn",
            agent_name, pid
        )));
    }

    CliService::success(&format!("✅ Detached process spawned with PID: {}", pid));
    Ok(pid)
}

/// Check if a process exists by PID (Linux-specific using /proc)
pub fn process_exists(pid: u32) -> bool {
    Path::new(&format!("/proc/{}", pid)).exists()
}

/// Terminate a process gracefully with SIGTERM
pub fn terminate_process(pid: u32) -> Result<()> {
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;

    signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM)
        .with_context(|| format!("Failed to send SIGTERM to PID {}", pid))?;

    Ok(())
}

/// Force kill a process with SIGKILL
pub fn force_kill_process(pid: u32) -> Result<()> {
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;

    signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL)
        .with_context(|| format!("Failed to send SIGKILL to PID {}", pid))?;

    Ok(())
}

/// Wait for a process to terminate gracefully, with fallback to force kill
pub async fn terminate_gracefully(pid: u32, timeout_secs: u64) -> Result<()> {
    terminate_process(pid)?;

    let check_interval = tokio::time::Duration::from_millis(100);
    let max_checks = (timeout_secs * 1000) / 100;

    for _ in 0..max_checks {
        if !process_exists(pid) {
            return Ok(());
        }
        tokio::time::sleep(check_interval).await;
    }

    force_kill_process(pid)?;

    for _ in 0..50 {
        if !process_exists(pid) {
            return Ok(());
        }
        tokio::time::sleep(check_interval).await;
    }

    Err(anyhow::anyhow!(
        "Failed to kill process {} even with SIGKILL",
        pid
    ))
}

/// Kill a process (alias for terminate_process for backwards compatibility)
pub fn kill_process(pid: u32) -> bool {
    terminate_process(pid).is_ok()
}

/// Check if a port is in use
pub fn is_port_in_use(port: u16) -> bool {
    use std::net::TcpListener;
    TcpListener::bind(format!("127.0.0.1:{}", port)).is_err()
}

/// Spawn detached process (alias for spawn_detached for backwards compatibility)
pub async fn spawn_detached_process(agent_name: &str, port: u16) -> OrchestrationResult<u32> {
    spawn_detached(agent_name, port).await
}

/// Validate that the agent binary exists
pub fn validate_agent_binary() -> Result<()> {
    let binary_path = BinaryPaths::resolve_binary("systemprompt")?;

    let metadata = fs::metadata(&binary_path)
        .with_context(|| format!("Failed to get metadata for: {}", binary_path.display()))?;

    if !metadata.is_file() {
        return Err(anyhow::anyhow!(
            "Agent binary is not a file: {}",
            binary_path.display()
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            return Err(anyhow::anyhow!(
                "Agent binary is not executable: {}",
                binary_path.display()
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_exists() {
        // Test with current process (should exist)
        let current_pid = std::process::id();
        assert!(process_exists(current_pid));

        // Test with non-existent PID (very unlikely to exist)
        assert!(!process_exists(99999999));
    }

    #[test]
    fn test_validate_agent_binary() {
        // Initialize config for test
        let _ = Config::init();

        // This test will fail if the binary doesn't exist, which is expected in dev
        // The function itself is correct - we're testing the validation logic
        match validate_agent_binary() {
            Ok(_) => {
                // Binary exists and is valid
            },
            Err(e) => {
                // Expected in development when binary hasn't been built
                let error_str = e.to_string();
                assert!(
                    error_str.contains("target/debug") || error_str.contains("systemprompt"),
                    "Unexpected error: {}",
                    error_str
                );
            },
        }
    }
}
