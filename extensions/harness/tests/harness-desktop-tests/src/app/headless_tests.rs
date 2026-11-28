use std::sync::Arc;

use systemprompt_device::secure::MemorySealer;
use systemprompt_harness::command::{ExitStatus, Streams};
use systemprompt_harness_views::app::headless::{self, Invocation};
use systemprompt_harness_views::app::paths::AppPaths;
use systemprompt_traits::CancellationToken;

use crate::TestResult;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|arg| (*arg).to_owned()).collect()
}

#[test]
fn no_arguments_open_the_window() {
    assert_eq!(headless::invocation(Vec::new()), Invocation::Window);
}

#[test]
fn a_directory_or_a_link_opens_the_window() {
    assert_eq!(
        headless::invocation(args(&["/work/project"])),
        Invocation::Window
    );
    assert_eq!(
        headless::invocation(args(&["harness://open?x=1"])),
        Invocation::Window
    );
    assert_eq!(headless::invocation(args(&["--run"])), Invocation::Window);
    assert_eq!(
        headless::invocation(args(&["/work", "run"])),
        Invocation::Window
    );
}

#[test]
fn run_and_login_are_subcommands_with_their_arguments() {
    let run = args(&["run", "--workspace", "/w", "--prompt", "hi"]);
    assert_eq!(headless::invocation(run.clone()), Invocation::Command(run));
    let login = args(&["login", "--stdin", "--instance", "http://127.0.0.1:1"]);
    assert_eq!(
        headless::invocation(login.clone()),
        Invocation::Command(login)
    );
}

#[test]
fn the_host_runs_over_the_window_data_root() -> TestResult {
    let dir = tempfile::tempdir()?;
    let paths = AppPaths::under(dir.path().join("root"), dir.path().join("home"));
    let identity = systemprompt_harness_brand::identity();
    let host = headless::host(identity, &paths, None, None);
    assert_eq!(host.data_root, paths.root);
    assert_eq!(host.home, paths.home);
    assert_eq!(host.info.product, identity.product());
    assert_eq!(host.project_dir, identity.config_dir_name());
    assert!(host.verdicts_enabled);
    assert!(!host.registrations.is_empty());
    let root = host.root(None);
    assert_eq!(root.store_file(), paths.store_file());
    assert_eq!(root.accounts_file(), paths.accounts_file());
    Ok(())
}

#[tokio::test]
async fn a_usage_error_goes_to_stderr_with_exit_64() -> TestResult {
    let dir = tempfile::tempdir()?;
    let paths = AppPaths::under(dir.path().join("root"), dir.path().join("home"));
    let mut host = headless::host(systemprompt_harness_brand::identity(), &paths, None, None);
    host.sealer = Arc::new(MemorySealer::for_tests());
    let mut output = Vec::new();
    let mut errors = Vec::new();
    let streams = Streams {
        input: &b""[..],
        output: &mut output,
        errors: &mut errors,
    };
    let status = systemprompt_harness::command::execute(
        &host,
        args(&["login", "--instance", "http://127.0.0.1:1"]),
        streams,
        &CancellationToken::new(),
    )
    .await;
    assert_eq!(status, ExitStatus::Usage);
    assert!(output.is_empty());
    assert!(String::from_utf8(errors)?.contains("login --stdin"));
    Ok(())
}
