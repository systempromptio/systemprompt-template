//! Runs the `systemprompt` CLI on behalf of an MCP tool call.
//!
//! Models routinely append flags the CLI does not accept; those are stripped
//! before exec rather than surfaced as a usage error the model cannot act on.
//!
//! Both halves of the location are validated up front. A bare `Command::spawn`
//! reports a missing binary and a missing working directory as the same
//! `ENOENT`, so neither could be told from the other at the call site.

use crate::tools::CliOutput;
use rmcp::ErrorData as McpError;
use std::path::PathBuf;
use systemprompt::config::ProfileBootstrap;
use systemprompt::models::paths::BuildPaths;
use systemprompt::models::profile::PathsConfig;
use tokio::process::Command;

/// Where the CLI lives and what directory it runs in.
///
/// Resolved once by the caller and passed down, rather than read from the
/// environment at each call: the environment is process-global, so a test that
/// pointed it at a stand-in binary changed the binary every other test in the
/// process would spawn.
#[derive(Debug)]
pub struct CliLocation {
    pub bin: PathBuf,
    pub workdir: PathBuf,
}

impl CliLocation {
    pub fn from_profile() -> Result<Self, McpError> {
        let profile = ProfileBootstrap::get()
            // Why: lint-ok: error-adapt — rmcp's ErrorData is a variant-less wire type
            .map_err(|e| McpError::internal_error(format!("Failed to get profile: {e}"), None))?;

        Self::from_paths(&profile.paths)
    }

    // Why: split from `from_profile` so the resolution is reachable without the
    // process-global profile bootstrap, which no test process has.
    #[doc(hidden)]
    pub fn from_paths(paths: &PathsConfig) -> Result<Self, McpError> {
        // Why: the profile pins exactly one of target/{debug,release}, but the CLI is
        // routinely built into only the other one. resolve_binary falls back to the
        // sibling directory and prefers the newer build, and names every path it tried
        // when neither holds the binary.
        let bin = BuildPaths::from_profile(paths)
            .resolve_binary("systemprompt")
            // Why: lint-ok: error-adapt — rmcp's ErrorData is a variant-less wire type
            .map_err(|e| {
                McpError::internal_error(
                    format!("Failed to locate the systemprompt CLI: {e}"),
                    None,
                )
            })?;

        let workdir = PathBuf::from(&paths.system);
        if !workdir.is_dir() {
            return Err(McpError::internal_error(
                format!(
                    "The profile's system directory does not exist, so the CLI has nowhere to run: {}",
                    workdir.display()
                ),
                None,
            ));
        }

        Ok(Self { bin, workdir })
    }
}

// Why: Strip CLI flags that models routinely hallucinate onto `systemprompt`
// invocations (output-format toggles the gateway sets itself). Exposed behind
// `#[doc(hidden)]` so the external test workspace can assert the filter set;
// not part of the public API.
#[doc(hidden)]
pub fn filter_hallucinated_args(args: Vec<String>) -> Vec<String> {
    const HALLUCINATED_ARGS: &[&str] = &["--json", "--output-format", "--format"];

    args.into_iter()
        .filter(|arg| !HALLUCINATED_ARGS.contains(&arg.as_str()))
        .collect()
}

pub(crate) async fn execute(
    location: &CliLocation,
    command: &str,
    auth_token: &str,
) -> Result<CliOutput, McpError> {
    let cli_path = &location.bin;
    let workdir = &location.workdir;

    // Why: lint-ok: error-adapt — rmcp's ErrorData is a variant-less wire type
    let args = shell_words::split(command).map_err(|e| {
        McpError::invalid_params(format!("Failed to parse command arguments: {e}"), None)
    })?;

    let args = filter_hallucinated_args(args);

    tracing::info!(
        cli_path = %cli_path.display(),
        workdir = %workdir.display(),
        args = ?args,
        "Executing CLI command"
    );

    let output = Command::new(cli_path)
        .args(&args)
        .env("SYSTEMPROMPT_NON_INTERACTIVE", "1")
        .env("SYSTEMPROMPT_OUTPUT_FORMAT", "json")
        .env("SYSTEMPROMPT_AUTH_TOKEN", auth_token)
        .current_dir(workdir)
        .output()
        .await
        // Why: lint-ok: error-adapt — rmcp's ErrorData is a variant-less wire type
        .map_err(|e| {
            McpError::internal_error(
                format!(
                    "Failed to execute CLI command: {e} (binary {}, workdir {})",
                    cli_path.display(),
                    workdir.display()
                ),
                None,
            )
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
