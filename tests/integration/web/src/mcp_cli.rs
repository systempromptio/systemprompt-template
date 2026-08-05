//! The `systemprompt` MCP tool driven end to end against a stand-in CLI.
//!
//! `SystempromptToolHandler::handle` shells out to whatever binary
//! `SYSTEMPROMPT_CLI_PATH` names, so pointing that variable at a shell script
//! written into a tempdir makes every branch of `cli::execute` and the handler
//! above it reachable without running the real CLI against the machine's
//! profile: the spawn failure, the argument-parse failure, the non-zero exit,
//! and both artifact arms (stdout that deserialises into a `CliArtifact` and
//! stdout that does not).
//!
//! The environment is process-global, so these tests rely on nextest's
//! process-per-test execution: each one owns its process and cannot race
//! another. The variable is read inside `cli::execute` on every call, so
//! setting it before dispatch is enough.

use std::sync::Arc;

use rmcp::model::CallToolRequestParams;
use sqlx::PgPool;
use systemprompt::database::Database;
use systemprompt::identifiers::{AgentName, ContextId, SessionId, TraceId};
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::{McpArtifactRepository, McpToolExecutor};
use systemprompt::models::artifacts::{CliArtifact, TextArtifact};
use systemprompt::models::execution::context::RequestContext as SysRequestContext;
use systemprompt_mcp_agent::filter_hallucinated_args;

use crate::tempdb::TempDb;

fn executor(pool: &Arc<PgPool>) -> McpToolExecutor {
    let db_pool = Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ));
    let usage = Arc::new(ToolUsageRepository::new(&db_pool).expect("tool usage repository"));
    let artifacts = Arc::new(McpArtifactRepository::new(&db_pool).expect("artifact repository"));
    McpToolExecutor::new(usage, artifacts, "systemprompt")
}

fn request_context() -> SysRequestContext {
    SysRequestContext::new(
        SessionId::new("cli-session"),
        TraceId::new("cli-trace"),
        ContextId::new("00000000-0000-4000-8000-00000000c11e"),
        AgentName::new("cli-agent"),
    )
}

// Why: `set_var` is unsafe from edition 2024 because another thread may be
// reading the environment concurrently. Under nextest each test is its own
// process and sets the variable before spawning anything, so there is no
// concurrent reader.
fn set_env(key: &str, value: &str) {
    unsafe { std::env::set_var(key, value) };
}

/// Write an executable `/bin/sh` script and point the CLI path at it.
fn fake_cli(dir: &tempfile::TempDir, body: &str) {
    let path = dir.path().join("systemprompt");
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("write the stand-in CLI");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("make the stand-in CLI executable");
    }
    set_env("SYSTEMPROMPT_CLI_PATH", &path.to_string_lossy());
}

fn call(command: &str) -> CallToolRequestParams {
    let arguments = serde_json::json!({ "command": command })
        .as_object()
        .expect("tool arguments are a JSON object")
        .clone();
    CallToolRequestParams::new("systemprompt").with_arguments(arguments)
}

async fn run(db: &TempDb, command: &str) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
    systemprompt_mcp_agent::server::tool::dispatch_tool(
        &executor(&db.pool),
        "systemprompt",
        &call(command),
        &request_context(),
        "test-bearer-token",
    )
    .await
}

fn body_of(result: &rmcp::model::CallToolResult) -> String {
    result
        .structured_content
        .as_ref()
        .and_then(|v| v.pointer("/artifact/content"))
        .and_then(|v| v.as_str())
        .expect("the executor returns the handler's artifact as structured content")
        .to_owned()
}

fn summary_of(result: &rmcp::model::CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|block| block.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_output_format_flags_models_invent_are_stripped_before_exec() {
    let filtered = filter_hallucinated_args(
        ["core", "skills", "list", "--json", "--output-format", "--format"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect(),
    );

    assert_eq!(
        filtered,
        vec!["core", "skills", "list"],
        "only the three output-format toggles are dropped"
    );
}

#[test]
fn arguments_that_are_not_hallucinated_flags_survive_the_filter() {
    let filtered = filter_hallucinated_args(
        ["plugins", "run", "discord", "send", "--channel", "42"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect(),
    );

    assert_eq!(filtered.len(), 6, "a real flag and its value both survive");
}

#[tokio::test]
async fn stdout_that_deserialises_into_an_artifact_is_returned_as_that_artifact() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let artifact =
        CliArtifact::text(TextArtifact::new("rendered from the CLI").with_title("Skills"));
    let encoded = serde_json::to_string(&artifact).expect("the artifact serialises");
    fake_cli(&dir, &format!("cat <<'ARTIFACT'\n{encoded}\nARTIFACT"));

    let result = run(&db, "core skills list")
        .await
        .expect("a zero-exit CLI call succeeds");

    assert_eq!(
        body_of(&result),
        "rendered from the CLI",
        "the handler returned the artifact the CLI emitted, not its JSON encoding"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn stdout_that_is_not_an_artifact_falls_back_to_a_text_artifact() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("tempdir");
    fake_cli(&dir, "printf 'plain human output'");

    let result = run(&db, "core skills list")
        .await
        .expect("a zero-exit CLI call succeeds");

    assert_eq!(
        body_of(&result),
        "plain human output",
        "unparseable stdout becomes the body of a text artifact"
    );
    assert_eq!(
        summary_of(&result),
        "plain human output",
        "the summary the model reads is the raw stdout either way"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_non_zero_exit_reports_the_code_and_the_stderr() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("tempdir");
    fake_cli(&dir, "echo 'no such skill' >&2\nexit 3");

    let error = run(&db, "core skills show nope")
        .await
        .expect_err("a non-zero exit is an error, not an artifact");

    assert!(
        error.message.contains("exit code 3"),
        "the failure names the exit code: {}",
        error.message
    );
    assert!(
        error.message.contains("no such skill"),
        "the failure carries the CLI's stderr: {}",
        error.message
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_cli_path_that_does_not_exist_is_reported_as_a_spawn_failure() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("tempdir");
    set_env(
        "SYSTEMPROMPT_CLI_PATH",
        &dir.path().join("absent").to_string_lossy(),
    );

    let error = run(&db, "core skills list")
        .await
        .expect_err("a missing binary cannot be executed");

    assert!(
        error.message.contains("Failed to execute CLI command"),
        "the failure distinguishes a spawn failure from a CLI error: {}",
        error.message
    );

    db.cleanup().await;
}

#[tokio::test]
async fn an_unbalanced_quote_is_refused_before_anything_is_spawned() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("tempdir");
    fake_cli(&dir, "echo 'this must never run'\nexit 1");

    let error = run(&db, "core skills show \"unterminated")
        .await
        .expect_err("a command that does not tokenise is refused");

    assert!(
        error.message.contains("Failed to parse command arguments"),
        "the argument parse failure is reported as such: {}",
        error.message
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_caller_token_and_the_non_interactive_flags_reach_the_process() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("tempdir");
    fake_cli(
        &dir,
        "printf '%s|%s|%s' \"$SYSTEMPROMPT_AUTH_TOKEN\" \"$SYSTEMPROMPT_NON_INTERACTIVE\" \
         \"$SYSTEMPROMPT_OUTPUT_FORMAT\"",
    );

    let result = run(&db, "core skills list")
        .await
        .expect("a zero-exit CLI call succeeds");

    assert_eq!(
        body_of(&result),
        "test-bearer-token|1|json",
        "the bearer token is forwarded, and the CLI is pinned to non-interactive JSON"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_hallucinated_flags_never_reach_the_spawned_process() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("tempdir");
    fake_cli(&dir, "printf '%s' \"$*\"");

    let result = run(&db, "core skills list --json --format --output-format")
        .await
        .expect("a zero-exit CLI call succeeds");

    assert_eq!(
        body_of(&result),
        "core skills list",
        "the filter runs between tokenising and spawning"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_process_runs_in_the_configured_workdir() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let workdir = tempfile::tempdir().expect("workdir");
    let canonical = workdir.path().canonicalize().expect("canonical workdir");
    fake_cli(&dir, "printf '%s' \"$(pwd -P)\"");
    set_env("SYSTEMPROMPT_WORKDIR", &canonical.to_string_lossy());

    let result = run(&db, "core skills list")
        .await
        .expect("a zero-exit CLI call succeeds");

    assert_eq!(
        body_of(&result),
        canonical.to_string_lossy(),
        "the CLI is spawned in the configured working directory"
    );

    db.cleanup().await;
}

// This test deliberately sets no `SYSTEMPROMPT_CLI_PATH`: with no profile
// bootstrapped in the test process, resolving the binary from the profile's
// bin directory is the failure the handler must surface.
#[tokio::test]
async fn without_the_env_override_the_path_comes_from_the_profile() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let error = run(&db, "core skills list")
        .await
        .expect_err("no profile is bootstrapped in a test process");

    assert!(
        error.message.contains("Failed to get profile"),
        "the missing profile is named as the reason the CLI could not be located: {}",
        error.message
    );

    db.cleanup().await;
}
