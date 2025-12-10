use anyhow::Result;
use std::process::Command;
use systemprompt_core_logging::CliService;

pub async fn terminate_gracefully(pid: u32) -> Result<()> {
    CliService::info(&format!("🔄 Sending SIGTERM to PID {pid}"));

    let output = Command::new("kill")
        .args(["-15", &pid.to_string()])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CliService::warning(&format!("⚠️ Failed to send SIGTERM to PID {pid}: {stderr}"));
    }

    Ok(())
}

pub async fn force_kill(pid: u32) -> Result<()> {
    CliService::info(&format!("⚡ Force killing PID {pid}"));

    let output = Command::new("kill")
        .args(["-9", &pid.to_string()])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CliService::warning(&format!("⚠️ Failed to force kill PID {pid}: {stderr}"));
    }

    Ok(())
}

pub async fn cleanup_port_processes(port: u16) -> Result<Vec<u32>> {
    CliService::info(&format!("🧹 Cleaning up processes on port {port}"));

    let output = Command::new("lsof")
        .args(["-ti", &format!(":{port}")])
        .output()?;

    if output.stdout.is_empty() {
        return Ok(vec![]);
    }

    let pids_string = String::from_utf8_lossy(&output.stdout);
    let mut killed_pids = Vec::new();

    for pid_str in pids_string.lines() {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            CliService::info(&format!(
                "🧹 Stopping process blocking port {port} (PID: {pid})"
            ));

            // Try graceful termination first
            terminate_gracefully(pid).await?;

            // Brief wait for graceful shutdown
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // Force kill if still running
            force_kill(pid).await?;

            killed_pids.push(pid);
        }
    }

    // Give time for port to be released
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    Ok(killed_pids)
}
