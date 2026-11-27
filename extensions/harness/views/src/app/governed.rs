//! The systemprompt side of the app, built inside the runtime in two steps
//! around the engine.
//!
//! `prepare` builds governance and installs it as the engine's Governor with
//! the verdict preference applied, inserts it into the shared handles, builds
//! the extension context, discovers the instance-side extensions and lets each
//! provide its handles (dependencies first), so the engine can be built from
//! their engine halves. `start` then starts the live sync, which hands every
//! applied instance to the extensions' manifest appliers, and runs the
//! extensions' background services while their extensions are admitted; the
//! shell context shares its settings, handles and enablement.

use std::sync::Arc;

use systemprompt_device::accounts::Accounts;
use systemprompt_device::client::HttpClient;
use systemprompt_engine::config::Layout;
use systemprompt_engine::run::extension::ExtensionSet;
use systemprompt_engine::run::{Engine, Governor};
use systemprompt_engine::store::Store;
use systemprompt_harness::extension::{
    Enablement, ExtensionError, Handles, HostInfo, SettingsAccess, SpContext, SpExtensions, SpPaths,
};
use systemprompt_harness::governance::Governance;
use systemprompt_harness::projection::Applier;
use systemprompt_harness::sync::{SyncDeps, SyncEngine, SyncError};
use systemprompt_harness_brand::Identity;
use systemprompt_harness_shared::contract::ShellContext;
use systemprompt_traits::{BackgroundTasks, CancellationToken};
use tokio::runtime::Handle;

use crate::app::governor::GovernorSlot;
use crate::app::paths::AppPaths;
use crate::app::settings_store::SettingsStore;
use crate::app::{extension_services, linked};

pub(crate) struct Governed {
    pub(crate) governance: Arc<Governance>,
    pub(crate) sync: SyncEngine,
    pub(crate) context: SpContext,
}

pub(crate) struct ShellParts {
    pub(crate) identity: &'static Identity,
    pub(crate) runtime: Handle,
    pub(crate) engine: Engine,
    pub(crate) store: Store,
    pub(crate) accounts: Accounts,
    pub(crate) http: HttpClient,
}

impl Governed {
    pub(crate) fn shell_context(&self, parts: ShellParts) -> ShellContext {
        let ShellParts {
            identity,
            runtime,
            engine,
            store,
            accounts,
            http,
        } = parts;
        ShellContext {
            identity,
            runtime,
            engine,
            store,
            accounts,
            http,
            paths: self.context.paths.clone(),
            settings: Arc::clone(&self.context.settings),
            handles: self.context.handles.clone(),
            enablement: self.context.enablement.clone(),
        }
    }
}

pub(crate) struct Prepared {
    governance: Arc<Governance>,
    context: SpContext,
    extensions: SpExtensions,
}

pub(crate) struct PrepareParts<'a> {
    pub(crate) identity: &'static Identity,
    pub(crate) paths: &'a AppPaths,
    pub(crate) layout: &'a Layout,
    pub(crate) accounts: &'a Accounts,
    pub(crate) http: &'a HttpClient,
    pub(crate) settings: &'a SettingsStore,
    pub(crate) governor: &'a GovernorSlot,
    pub(crate) cancel: &'a CancellationToken,
}

pub(crate) fn prepare(parts: &PrepareParts<'_>) -> Result<Prepared, ExtensionError> {
    let governance = Governance::new(parts.cancel);
    governance.set_verdicts_enabled(parts.settings.read(|settings| settings.verdict_enabled));
    let installed: Arc<dyn Governor> = Arc::<Governance>::clone(&governance);
    parts.governor.install(installed);

    let handles = Handles::default();
    handles.insert(Arc::clone(&governance));
    let settings: Arc<dyn SettingsAccess> = Arc::new(parts.settings.clone());
    let context = SpContext {
        host: HostInfo {
            product: parts.identity.product(),
            display_name: parts.identity.display_name(),
            version: parts.identity.version(),
        },
        paths: SpPaths {
            engine: parts.paths.engine_paths(),
            layout: parts.layout.clone(),
            state: parts.paths.state.clone(),
            cache: parts.paths.cache.clone(),
            documents: parts.paths.documents.clone(),
        },
        accounts: parts.accounts.clone(),
        http: parts.http.clone(),
        settings,
        handles,
        enablement: Enablement::new(),
    };
    let extensions = SpExtensions::registered()?;
    linked::report_missing(
        &linked::instance_side(),
        extensions.entries().iter().map(|entry| &entry.metadata),
        "instance",
    );
    extensions.provide(&context);
    Ok(Prepared {
        governance,
        context,
        extensions,
    })
}

pub(crate) struct StartParts<'a> {
    pub(crate) identity: &'static Identity,
    pub(crate) paths: &'a AppPaths,
    pub(crate) layout: &'a Layout,
    pub(crate) tasks: &'a BackgroundTasks,
    pub(crate) cancel: &'a CancellationToken,
}

impl Prepared {
    pub(crate) fn engine_set(&self) -> Result<ExtensionSet, ExtensionError> {
        self.extensions.engine_set(&self.context)
    }

    pub(crate) fn start(self, parts: &StartParts<'_>) -> Result<Governed, SyncError> {
        let Self {
            governance,
            context,
            extensions,
        } = self;
        let sync = SyncEngine::start(SyncDeps {
            accounts: context.accounts.clone(),
            governance: Arc::clone(&governance),
            appliers: extensions.appliers(&context),
            applier: Applier::new(parts.paths.engine_paths(), parts.layout.clone()),
            state_dir: parts.paths.state.clone(),
            cache_dir: parts.paths.cache.clone(),
            release_version: parts.identity.version().to_owned(),
            cancel: parts.cancel.child_token(),
        })?;
        context.handles.insert(Arc::new(sync.clone()));
        extension_services::spawn_all(parts.tasks, &extensions, &context);
        tracing::info!(
            extensions = extensions.entries().len(),
            "governance, the sync and the extensions started"
        );
        Ok(Governed {
            governance,
            sync,
            context,
        })
    }
}
