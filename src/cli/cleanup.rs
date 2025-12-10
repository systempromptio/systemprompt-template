use anyhow::{Context, Result};
use std::process::Command;
use std::sync::Arc;
use systemprompt_core_logging::CliService;
use systemprompt_core_system::AppContext;
use systemprompt_models::repository::ServiceRepository;

pub async fn execute() -> Result<()> {
    CliService::section("Cleaning Up Services");

    let ctx = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );

    let service_repo = ServiceRepository::new(ctx.db_pool().clone());

    CliService::info("🔍 Finding running services...");
    let running_services = service_repo.get_running_services_with_pid().await?;

    if running_services.is_empty() {
        CliService::info("  No running services found in database");
    } else {
        CliService::info(&format!(
            "  Found {} running service(s)",
            running_services.len()
        ));

        for service in &running_services {
            if let Some(pid) = service.pid {
                CliService::info(&format!(
                    "🛑 Stopping {} (PID: {}, port: {})...",
                    service.name, pid, service.port
                ));

                kill_process(pid);
                kill_port(service.port as u16);

                service_repo
                    .update_service_stopped(&service.name)
                    .await
                    .ok();
            }
        }
    }

    CliService::info("🛑 Stopping API server...");
    kill_port(8080);
    kill_by_name("systemprompt serve api");

    CliService::info("🔍 Verifying port 8080 is free...");
    verify_port_free(8080, 3)?;

    CliService::success("✅ All services cleaned up");
    Ok(())
}

fn kill_process(pid: i32) {
    Command::new("kill")
        .args(["-9", &pid.to_string()])
        .output()
        .ok();
}

fn kill_port(port: u16) {
    if port == 5432 || port == 6432 {
        return;
    }

    let output = Command::new("lsof")
        .args(["-ti", &format!(":{}", port)])
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
    if name.contains("postgres") || name.contains("pgbouncer") || name.contains("psql") {
        return;
    }

    Command::new("pkill").args(["-9", "-f", name]).output().ok();
}

pub fn check_port_available(port: u16) -> Option<i32> {
    let output = Command::new("lsof")
        .args(["-ti", &format!(":{}", port)])
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
                    "Port {} is still occupied by PID {} after {} attempts. Try running: kill -9 \
                     {} or just api-nuke",
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
