use anyhow::Result;
use std::process::Command;
use systemprompt_core_logging::CliService;

pub async fn execute() -> Result<()> {
    CliService::section("Cleaning Up Services");

    // Note: Skip database lookups to avoid prepared statement conflicts with PgBouncer
    // Just clean up common service ports instead
    CliService::info("ℹ  Cleaning up common service ports...");

    CliService::info("🛑 Stopping API server...");
    kill_port(8080);
    kill_by_name("systemprompt serve api");

    // Note: Skip database cleanup to avoid prepared statement conflicts with PgBouncer
    // Database will be cleaned up on next migration or manual command
    CliService::info("ℹ  (Skipping database cleanup to preserve connection pool)");

    CliService::info("🔍 Verifying port 8080 is free...");
    verify_port_free(8080, 3)?;

    CliService::success("✅ All services cleaned up");
    Ok(())
}

fn kill_process(pid: i32) {
    Command::new("kill")
        .args(&["-9", &pid.to_string()])
        .output()
        .ok();
}

fn kill_port(port: u16) {
    // Don't kill database-related ports
    if port == 5432 || port == 6432 {
        return;
    }

    let output = Command::new("lsof")
        .args(&["-ti", &format!(":{}", port)])
        .output();

    if let Ok(output) = output {
        let pids = String::from_utf8_lossy(&output.stdout);
        for pid in pids.lines() {
            if let Ok(pid_num) = pid.trim().parse::<i32>() {
                kill_process(pid_num);
            }
        }
    }
}

fn kill_by_name(name: &str) {
    // Don't kill database-related processes
    if name.contains("postgres") || name.contains("pgbouncer") || name.contains("psql") {
        return;
    }

    Command::new("pkill")
        .args(&["-9", "-f", name])
        .output()
        .ok();
}

pub fn check_port_available(port: u16) -> Option<i32> {
    let output = Command::new("lsof")
        .args(&["-ti", &format!(":{}", port)])
        .output()
        .ok()?;

    if output.stdout.is_empty() {
        None
    } else {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .and_then(|pid| pid.trim().parse::<i32>().ok())
    }
}

fn verify_port_free(port: u16, max_retries: u8) -> Result<()> {
    for attempt in 1..=max_retries {
        if let Some(pid) = check_port_available(port) {
            if attempt < max_retries {
                CliService::warning(&format!(
                    "  ⏳ Port {} still occupied by PID {}, retry {}/{}...",
                    port, pid, attempt, max_retries
                ));
                std::thread::sleep(std::time::Duration::from_secs(1));
            } else {
                return Err(anyhow::anyhow!(
                    "Port {} is still occupied by PID {} after {} attempts. \
                     Try running: kill -9 {} or just api-nuke",
                    port,
                    pid,
                    max_retries,
                    pid
                ));
            }
        } else {
            CliService::info(&format!("  ✅ Port {} is free", port));
            return Ok(());
        }
    }
    Ok(())
}
