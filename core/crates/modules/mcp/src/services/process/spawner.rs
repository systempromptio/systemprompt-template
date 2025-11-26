use super::ProcessManager;
use crate::McpServerConfig;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use systemprompt_core_logging::CliService;
use systemprompt_core_system::BinaryPaths;

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

pub async fn spawn_server(_manager: &ProcessManager, config: &McpServerConfig) -> Result<u32> {
    let binary_path = BinaryPaths::resolve_binary(&config.name)
        .with_context(|| format!("Failed to find binary for {}", config.name))?;

    let config_global = systemprompt_core_system::Config::global();

    let log_dir = Path::new(&config_global.system_path).join("logs");
    fs::create_dir_all(&log_dir)
        .with_context(|| format!("Failed to create logs directory: {}", log_dir.display()))?;

    let log_file_path = log_dir.join(format!("mcp-{}.log", config.name));
    rotate_log_if_needed(&log_file_path)?;

    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .with_context(|| format!("Failed to create log file: {}", log_file_path.display()))?;

    let child = Command::new(&binary_path)
        .env("DATABASE_URL", &config_global.database_url)
        .env("DATABASE_TYPE", &config_global.database_type)
        .env("API_SERVER_URL", &config_global.api_server_url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::from(log_file))
        .stdin(std::process::Stdio::null())
        .spawn()
        .with_context(|| format!("Failed to start detached {}", config.name))?;

    let pid = child.id();

    std::mem::forget(child);

    Ok(pid)
}

pub async fn verify_binary(config: &McpServerConfig) -> Result<()> {
    let binary_path = BinaryPaths::resolve_binary(&config.name)?;

    let metadata = fs::metadata(&binary_path)
        .with_context(|| format!("Binary not found: {}", binary_path.display()))?;

    CliService::success(&format!(
        "✅ Binary verified: {} ({} bytes)",
        binary_path.display(),
        metadata.len()
    ));
    Ok(())
}

pub async fn build_server(config: &McpServerConfig) -> Result<()> {
    use systemprompt_core_system::Config;
    let cargo_target_dir = &Config::global().cargo_target_dir;

    CliService::info(&format!(
        "🔨 Building service: {} (debug mode)",
        config.name
    ));

    let output = Command::new("cargo")
        .env("CARGO_TARGET_DIR", cargo_target_dir)
        .args(["build", "--package", &config.name, "--bin", &config.name])
        .output()
        .with_context(|| format!("Failed to build {}", config.name))?;

    if output.status.success() {
        CliService::success(&format!("✅ Built {} (debug)", config.name));
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CliService::error(&format!("❌ Build failed for {}: {}", config.name, stderr));
        Err(anyhow::anyhow!("Build failed for {}", config.name))
    }
}
