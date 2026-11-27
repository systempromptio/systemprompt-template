//! The native shell and composition root: reads the launch, becomes (or hands
//! off to) the running instance, starts the services on one tokio runtime,
//! opens the window and shuts everything down when the app quits.
//!
//! The `harness` binary calls [`run`]; the modules are public so the test
//! workspace reaches what it tests.

pub mod app;
pub mod keymap;
pub mod state;
pub mod strings;
pub mod views;

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use anyhow::Context as _;
use tokio::runtime::Runtime;

use crate::app::deeplink::{self, LaunchRequest};
use crate::app::paths::{self, AppPaths, Forbidden};
use crate::app::settings_store::SettingsStore;
use crate::app::single_instance::{self, Endpoint, Instance};
use crate::app::window::{self, Launch};
use crate::app::{Services, ServicesConfig, logging};

const SHUTDOWN_WAIT: Duration = Duration::from_secs(5);
const RUNTIME_DRAIN: Duration = Duration::from_secs(2);
const RUNTIME_THREAD: &str = "harness-runtime";

pub fn run() -> ExitCode {
    match start() {
        Ok(code) => code,
        Err(error) => {
            tracing::error!(error = ?error, "startup failed");
            ExitCode::FAILURE
        },
    }
}

fn start() -> anyhow::Result<ExitCode> {
    let identity = sp_brand::identity();
    let home = dirs::home_dir().unwrap_or_else(|| identity.data_root());
    let paths = AppPaths {
        documents: dirs::document_dir(),
        ..AppPaths::under(identity.data_root(), home)
    };
    paths.ensure().context("creating the data directories")?;
    // Why: discard-ok: without a log file the failure has nowhere to go.
    let _log = logging::init(&paths.logs, identity.scheme()).ok();

    let mut request =
        deeplink::parse_args(identity.scheme(), launch_arguments()).unwrap_or_else(|error| {
            tracing::warn!(error = %error, "command line not understood");
            LaunchRequest::default()
        });
    request.directory = request.directory.as_deref().and_then(paths::absolute);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name(RUNTIME_THREAD)
        .build()
        .context("starting the async runtime")?;
    let endpoint = Endpoint::for_app(identity, &paths);
    let instance = runtime.block_on(single_instance::acquire(&endpoint, &request));
    if matches!(instance, Instance::Forwarded) {
        tracing::info!("handed the start to the running app");
        return Ok(ExitCode::SUCCESS);
    }

    let settings = SettingsStore::load(paths.ui_settings_file());
    let directory = launch_directory(&request, &paths, &settings);
    let config = ServicesConfig {
        identity,
        paths,
        runtime: runtime.handle().clone(),
        settings,
        search_path: std::env::var_os("PATH"),
        path_ext: std::env::var_os("PATHEXT"),
    };
    let services = match Services::start(config) {
        Ok(services) => services,
        Err(error) => {
            tracing::error!(error = ?error, "services did not start");
            window::show_fatal();
            return Ok(ExitCode::FAILURE);
        },
    };

    let (sender, requests) = tokio::sync::mpsc::unbounded_channel();
    if let Instance::Primary(listener) = instance {
        let _entered = runtime.enter();
        single_instance::serve(listener, endpoint, services.tasks(), sender.clone());
    }
    window::run(Launch {
        services: services.clone(),
        directory,
        links: request.links,
        forward: sender,
        requests,
    });
    stop(&runtime, &services);
    runtime.shutdown_timeout(RUNTIME_DRAIN);
    Ok(ExitCode::SUCCESS)
}

fn launch_arguments() -> impl Iterator<Item = String> {
    std::env::args_os()
        .skip(1)
        .map(|argument| argument.to_string_lossy().into_owned())
}

fn launch_directory(
    request: &LaunchRequest,
    paths: &AppPaths,
    settings: &SettingsStore,
) -> PathBuf {
    let forbidden = Forbidden {
        system_root: std::env::var_os("SystemRoot").map(PathBuf::from),
        install_dir: std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf)),
    };
    let scheme = sp_brand::identity().scheme();
    let requested = request
        .project_directory(scheme)
        .and_then(|directory| paths::absolute(&directory));
    let last = settings.read(|settings| settings.last_project.clone());
    paths::choose_directory(requested, last, &paths.home, &forbidden, Path::is_dir)
}

fn stop(runtime: &Runtime, services: &Services) {
    let services = services.clone();
    let finished = runtime
        .block_on(async move { tokio::time::timeout(SHUTDOWN_WAIT, services.shutdown()).await });
    if finished.is_err() {
        tracing::warn!("shutdown did not finish in time");
    }
}
