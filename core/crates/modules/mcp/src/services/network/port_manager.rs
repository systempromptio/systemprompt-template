use anyhow::Result;
use std::process::Command;
use systemprompt_core_logging::CliService;

pub async fn prepare_port(port: u16) -> Result<()> {
    CliService::info(&format!("🔍 Preparing port {port}..."));

    // Check if port is already in use
    if is_port_in_use(port).await? {
        CliService::info(&format!("🧹 Port {port} is in use, cleaning up..."));
        cleanup_port_processes(port).await?;
    }

    CliService::success(&format!("✅ Port {port} is ready"));
    Ok(())
}

pub async fn is_port_in_use(port: u16) -> Result<bool> {
    match std::net::TcpStream::connect(format!("127.0.0.1:{port}")) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub async fn is_port_responsive(port: u16) -> Result<bool> {
    is_port_in_use(port).await
}

pub async fn cleanup_port_processes(port: u16) -> Result<()> {
    let output = Command::new("lsof")
        .args(["-ti", &format!(":{port}")])
        .output()?;

    if !output.stdout.is_empty() {
        let pids = String::from_utf8_lossy(&output.stdout);
        for pid in pids.lines() {
            if !pid.is_empty() {
                CliService::info(&format!(
                    "🧹 Stopping process on port {port} (PID: {pid})"
                ));

                // Try graceful termination
                let _ = Command::new("kill").args(["-15", pid]).output();

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                // Force kill if still running
                let _ = Command::new("kill").args(["-9", pid]).output();
            }
        }

        // Wait for port release
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    Ok(())
}

pub async fn wait_for_port_release(port: u16) -> Result<()> {
    let max_attempts = 10;
    let delay = std::time::Duration::from_millis(100);

    for attempt in 1..=max_attempts {
        if !is_port_in_use(port).await? {
            return Ok(());
        }

        if attempt < max_attempts {
            tokio::time::sleep(delay).await;
        }
    }

    Err(anyhow::anyhow!(
        "Port {port} did not become available after {max_attempts} attempts"
    ))
}

pub async fn cleanup_port_resources(_port: u16) -> Result<()> {
    // Additional cleanup if needed
    Ok(())
}

pub async fn find_available_port(start_port: u16, end_port: u16) -> Result<u16> {
    for port in start_port..=end_port {
        if !is_port_in_use(port).await? {
            return Ok(port);
        }
    }

    Err(anyhow::anyhow!(
        "No available ports in range {start_port}-{end_port}"
    ))
}
