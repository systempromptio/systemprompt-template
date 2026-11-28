//! The `run` and `login` subcommands: core's headless entry over the
//! window's host, the process's standard streams, Ctrl-C as cancellation and
//! the command's exit status as the process's.
//!
//! On Windows a release build is a GUI-subsystem program, which gets no console
//! of its own. Standard handles a parent passes (a pipe or a file, as the tour
//! and any script redirecting output use) are inherited and work unchanged;
//! writing to the console of an interactive shell would need
//! `AttachConsole(ATTACH_PARENT_PROCESS)`, which this crate does not call.

use std::ffi::OsString;
use std::process::ExitCode;

use systemprompt_harness_views::app::headless;
use systemprompt_harness_views::app::paths::AppPaths;
use systemprompt_traits::CancellationToken;
use tokio::runtime::Runtime;

pub(crate) struct Environment {
    pub(crate) search_path: Option<OsString>,
    pub(crate) path_ext: Option<OsString>,
}

pub(crate) fn execute(
    runtime: &Runtime,
    paths: &AppPaths,
    args: Vec<String>,
    environment: Environment,
) -> ExitCode {
    let identity = systemprompt_harness_brand::identity();
    let host = headless::host(
        identity,
        paths,
        environment.search_path,
        environment.path_ext,
    );
    let cancel = CancellationToken::new();
    let interrupted = cancel.clone();
    runtime.spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::info!("interrupted");
            interrupted.cancel();
        }
    });
    runtime.block_on(headless::execute(&host, args, &cancel))
}
