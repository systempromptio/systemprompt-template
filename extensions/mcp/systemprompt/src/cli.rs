use crate::tools::CliOutput;
use rmcp::ErrorData as McpError;
use std::path::PathBuf;
use std::process::Command;
use systemprompt::models::ProfileBootstrap;

pub fn get_cli_path() -> Result<PathBuf, McpError> {
    if let Ok(path) = std::env::var("SYSTEMPROMPT_CLI_PATH") {
        return Ok(PathBuf::from(path));
    }

    let profile = ProfileBootstrap::get()
        .map_err(|e| McpError::internal_error(format!("Failed to get profile: {e}"), None))?;

    Ok(PathBuf::from(&profile.paths.bin).join("systemprompt"))
}

pub fn get_workdir() -> PathBuf {
    if let Ok(path) = std::env::var("SYSTEMPROMPT_WORKDIR") {
        return PathBuf::from(path);
    }

    ProfileBootstrap::get().map_or_else(|_| PathBuf::from("."), |p| PathBuf::from(&p.paths.system))
}

/// Filter out common model hallucinations from command arguments.
/// Models sometimes add flags like --json even when not supported.
fn filter_hallucinated_args(args: Vec<String>) -> Vec<String> {
    const HALLUCINATED_ARGS: &[&str] = &["--json", "--output-format", "--format"];

    args.into_iter()
        .filter(|arg| !HALLUCINATED_ARGS.contains(&arg.as_str()))
        .collect()
}

pub fn execute(command: &str, auth_token: &str) -> Result<CliOutput, McpError> {
    let cli_path = get_cli_path()?;
    let workdir = get_workdir();

    let args = shell_words::split(command).map_err(|e| {
        McpError::invalid_params(format!("Failed to parse command arguments: {e}"), None)
    })?;

    // Filter out hallucinated arguments that models sometimes add
    let args = filter_hallucinated_args(args);

    tracing::info!(
        cli_path = %cli_path.display(),
        workdir = %workdir.display(),
        args = ?args,
        "Executing CLI command"
    );

    let output = Command::new(&cli_path)
        .args(&args)
        .env("SYSTEMPROMPT_NON_INTERACTIVE", "1")
        .env("SYSTEMPROMPT_OUTPUT_FORMAT", "json")
        .env("SYSTEMPROMPT_AUTH_TOKEN", auth_token)
        .current_dir(workdir)
        .output()
        .map_err(|e| {
            McpError::internal_error(format!("Failed to execute CLI command: {e}"), None)
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    let success = output.status.success();

    tracing::info!(
        exit_code = exit_code,
        success = success,
        stdout_len = stdout.len(),
        stderr_len = stderr.len(),
        "CLI command completed"
    );

    Ok(CliOutput {
        stdout,
        stderr,
        exit_code,
        success,
    })
}
