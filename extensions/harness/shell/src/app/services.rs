//! The long-lived handles every screen works with, built once at startup.
//!
//! `Services` is the composition root's output: cheap to clone, handed to every
//! state entity and view. It owns no UI state. Startup, inside the tokio
//! runtime: directories, the session store, the instance HTTP client, the
//! accounts, governance installed into the `GovernorSlot` and the
//! instance-side extensions provided, the engine built from their engine
//! halves, then the live sync and the extensions' services, and finally the
//! shell extensions and the `ShellContext` their views are built with. Screens
//! find shared handles (`Governance`, `SyncEngine` and whatever extensions
//! provide) in `Services::handles` by type. Shutdown runs in the reverse order
//! of dependence: sync, the owned background tasks (the extensions' services
//! among them), governance cleared, engine, accounts.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use harness_config::Layout;
use harness_engine::{Engine, EngineError, Governor};
use harness_llm::TransportError;
use harness_shell::{ShellContext, ShellExtensions};
use harness_store::{Store, StoreError};
use harness_tasks::{BackgroundTasks, CancellationToken};
use sp_accounts::{Accounts, AccountsError};
use sp_brand::Identity;
use sp_client::{HttpClient, HttpClientError};
use sp_extension::{Enablement, ExtensionError, Handles};
use sp_governance::Governance;
use sp_protocol::host::HOST_ID;
use sp_sync::{SyncEngine, SyncError};
use tokio::runtime::Handle;

use crate::app::governed::{self, PrepareParts, ShellParts, StartParts};
use crate::app::governor::GovernorSlot;
use crate::app::linked;
use crate::app::paths::AppPaths;
use crate::app::settings_store::SettingsStore;
use crate::app::startup::{self, EngineParts};

const TASKS_DRAIN: Duration = Duration::from_secs(3);

#[derive(Debug, thiserror::Error)]
pub(crate) enum StartupError {
    #[error("the directory {} could not be created", path.display())]
    Directories {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("the session store could not be opened")]
    Store(#[source] StoreError),
    #[error("the instance HTTP client could not be built")]
    InstanceClient(#[source] HttpClientError),
    #[error("the model HTTP client could not be built")]
    ModelClient(#[source] TransportError),
    #[error("the accounts could not be opened")]
    Accounts(#[source] AccountsError),
    #[error("the linked extensions could not be assembled")]
    Extensions(#[source] ExtensionError),
    #[error("the engine could not be started")]
    Engine(#[source] EngineError),
    #[error("the live sync could not be started")]
    Sync(#[source] SyncError),
}

#[derive(Debug)]
pub(crate) struct ServicesConfig {
    pub(crate) identity: &'static Identity,
    pub(crate) paths: AppPaths,
    pub(crate) runtime: Handle,
    pub(crate) settings: SettingsStore,
    pub(crate) search_path: Option<OsString>,
    pub(crate) path_ext: Option<OsString>,
}

#[derive(Clone)]
pub(crate) struct Services {
    inner: Arc<Inner>,
}

struct Inner {
    identity: &'static Identity,
    paths: AppPaths,
    layout: Layout,
    governor: Arc<GovernorSlot>,
    governance: Arc<Governance>,
    sync: SyncEngine,
    shell: ShellContext,
    shell_extensions: ShellExtensions,
    tasks: BackgroundTasks,
    settings: SettingsStore,
    shutdown: CancellationToken,
}

fn layout(identity: &Identity) -> Layout {
    Layout {
        project_dir: identity.config_dir_name().to_owned(),
        ..Layout::default()
    }
}

fn instance_client(identity: &Identity) -> Result<HttpClient, StartupError> {
    let user_agent = format!("{HOST_ID}/{}", identity.version());
    HttpClient::new(&user_agent).map_err(StartupError::InstanceClient)
}

fn ensure_directories(paths: &AppPaths) -> Result<(), StartupError> {
    paths.ensure().map_err(|source| StartupError::Directories {
        path: paths.root.clone(),
        source,
    })
}

fn discover_shell_extensions() -> Result<ShellExtensions, StartupError> {
    let extensions = ShellExtensions::discover().map_err(StartupError::Extensions)?;
    linked::report_missing(
        &linked::shell_side(),
        extensions.entries().iter().map(|entry| &entry.metadata),
        "shell",
    );
    Ok(extensions)
}

impl std::fmt::Debug for Services {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Services")
            .field("app", &self.inner.identity.display_name())
            .field("root", &self.inner.paths.root)
            .finish_non_exhaustive()
    }
}

impl Services {
    pub(crate) fn start(config: ServicesConfig) -> Result<Self, StartupError> {
        let ServicesConfig {
            identity,
            paths,
            runtime,
            settings,
            search_path,
            path_ext,
        } = config;
        let _entered = runtime.enter();
        ensure_directories(&paths)?;
        let layout = layout(identity);
        let store = Store::open(paths.store_file()).map_err(StartupError::Store)?;
        let http = instance_client(identity)?;
        let accounts = startup::open_accounts(identity, &paths, &http)?;
        let shutdown = CancellationToken::new();
        let tasks = BackgroundTasks::new();
        let governor = Arc::new(GovernorSlot::default());
        let shell_extensions = discover_shell_extensions()?;

        let prepared = governed::prepare(&PrepareParts {
            identity,
            paths: &paths,
            layout: &layout,
            accounts: &accounts,
            http: &http,
            settings: &settings,
            governor: &governor,
            cancel: &shutdown,
        })
        .map_err(StartupError::Extensions)?;
        let engine = startup::open_engine(EngineParts {
            identity,
            paths: &paths,
            layout: &layout,
            store: &store,
            governor: &governor,
            extensions: prepared.engine_set().map_err(StartupError::Extensions)?,
            search_path,
            path_ext,
        })?;
        let governed = prepared
            .start(&StartParts {
                identity,
                paths: &paths,
                layout: &layout,
                tasks: &tasks,
                cancel: &shutdown,
            })
            .map_err(StartupError::Sync)?;

        let shell = governed.shell_context(ShellParts {
            identity,
            runtime,
            engine,
            store,
            accounts,
            http,
        });

        settings.spawn_saver(&tasks);
        tracing::info!(app = identity.display_name(), root = %paths.root.display(), "services started");
        Ok(Self {
            inner: Arc::new(Inner {
                identity,
                paths,
                layout,
                governor,
                governance: governed.governance,
                sync: governed.sync,
                shell,
                shell_extensions,
                tasks,
                settings,
                shutdown,
            }),
        })
    }

    pub(crate) fn identity(&self) -> &'static Identity {
        self.inner.identity
    }

    pub(crate) fn paths(&self) -> &AppPaths {
        &self.inner.paths
    }

    pub(crate) fn layout(&self) -> &Layout {
        &self.inner.layout
    }

    pub(crate) fn runtime(&self) -> &Handle {
        &self.inner.shell.runtime
    }

    pub(crate) fn store(&self) -> &Store {
        &self.inner.shell.store
    }

    pub(crate) fn engine(&self) -> &Engine {
        &self.inner.shell.engine
    }

    pub(crate) fn accounts(&self) -> &Accounts {
        &self.inner.shell.accounts
    }

    pub(crate) fn settings(&self) -> &SettingsStore {
        &self.inner.settings
    }

    pub(crate) fn governor(&self) -> Arc<dyn Governor> {
        self.inner.governor.current()
    }

    pub(crate) fn sync(&self) -> &SyncEngine {
        &self.inner.sync
    }

    pub(crate) fn handles(&self) -> &Handles {
        &self.inner.shell.handles
    }

    pub(crate) fn enablement(&self) -> &Enablement {
        &self.inner.shell.enablement
    }

    pub(crate) fn shell(&self) -> &ShellContext {
        &self.inner.shell
    }

    pub(crate) fn shell_extensions(&self) -> &ShellExtensions {
        &self.inner.shell_extensions
    }

    pub(crate) fn tasks(&self) -> &BackgroundTasks {
        &self.inner.tasks
    }

    pub(crate) async fn shutdown(&self) {
        let inner = &self.inner;
        inner.sync.shutdown().await;
        if !inner.tasks.shutdown(TASKS_DRAIN).await.is_drained() {
            tracing::warn!("background tasks outlived shutdown");
        }
        inner.governance.clear();
        inner.shell.engine.shutdown().await;
        inner.shell.accounts.shutdown();
        inner.shutdown.cancel();
        if let Err(error) = inner.settings.flush() {
            tracing::warn!(error = %error, "preferences not saved at shutdown");
        }
        tracing::info!("services stopped");
    }
}
