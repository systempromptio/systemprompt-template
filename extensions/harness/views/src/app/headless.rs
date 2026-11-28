//! The headless half of the binary: which invocation an argument list asks
//! for, and the host the `run` and `login` subcommands run as.
//!
//! The host is built from the same identity, data root, preferences and
//! extension list the window starts with, so a sign-in made by `login` is the
//! window's and a `run` session appears in its session list. A subcommand is
//! the first argument naming `run` or `login` exactly; anything else, nothing
//! included, opens the window. `execute` runs one over the process's standard
//! streams and returns its exit status as the process's.

use std::ffi::OsString;
use std::io::BufReader;
use std::process::ExitCode;
use std::sync::Arc;

use systemprompt_harness::command::{HeadlessHost, Streams};
use systemprompt_harness::extension::{HostInfo, SettingsAccess};
use systemprompt_harness::members::REGISTERED;
use systemprompt_harness_brand::Identity;
use systemprompt_traits::CancellationToken;

use crate::app::paths::AppPaths;
use crate::app::settings_store::SettingsStore;

const SUBCOMMANDS: [&str; 2] = ["run", "login"];

/// What the process was started to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invocation {
    Window,
    Command(Vec<String>),
}

pub fn invocation(args: Vec<String>) -> Invocation {
    let named = args
        .first()
        .is_some_and(|first| SUBCOMMANDS.contains(&first.as_str()));
    if named {
        Invocation::Command(args)
    } else {
        Invocation::Window
    }
}

pub fn host(
    identity: &'static Identity,
    paths: &AppPaths,
    search_path: Option<OsString>,
    path_ext: Option<OsString>,
) -> HeadlessHost {
    let store = SettingsStore::load(paths.ui_settings_file());
    let verdicts_enabled = store.read(|settings| settings.verdict_enabled);
    let settings: Arc<dyn SettingsAccess> = Arc::new(store);
    HeadlessHost {
        info: HostInfo {
            product: identity.product(),
            display_name: identity.display_name(),
            version: identity.version(),
        },
        project_dir: identity.config_dir_name().to_owned(),
        device_name: identity.device_label(&systemprompt_device::accounts::computer_name()),
        data_root: paths.root.clone(),
        home: paths.home.clone(),
        documents: paths.documents.clone(),
        sealer: systemprompt_device::secure::platform_sealer(identity.app_id()),
        settings,
        registrations: REGISTERED.to_vec(),
        verdicts_enabled,
        search_path,
        path_ext,
    }
}

pub async fn execute(
    host: &HeadlessHost,
    args: Vec<String>,
    cancel: &CancellationToken,
) -> ExitCode {
    let streams = Streams {
        input: BufReader::new(std::io::stdin()),
        output: std::io::stdout(),
        errors: std::io::stderr(),
    };
    let status = systemprompt_harness::command::execute(host, args, streams, cancel).await;
    tracing::info!(status = ?status, "command finished");
    status.into()
}
