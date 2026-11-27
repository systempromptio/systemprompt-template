//! What the composition root hands every shell extension's views.

use std::fmt;
use std::sync::Arc;

use systemprompt_device::accounts::Accounts;
use systemprompt_device::client::HttpClient;
use systemprompt_engine::run::Engine;
use systemprompt_engine::store::Store;
use systemprompt_harness::extension::{
    Enablement, ExtensionSettings, Handles, SettingsAccess, SpPaths,
};
use systemprompt_harness_brand::Identity;
use systemprompt_identifiers::engine::ExtensionId;
use tokio::runtime::Handle;

/// Everything a shell extension's views work with, built once by the
/// composition root and cheap to clone.
///
/// `runtime` is the tokio runtime every engine, store and instance call runs
/// on (the view applies results on the UI thread); `paths` are the directories
/// the instance-side halves were given, the user's Documents directory among
/// them; `handles` carries what the instance-side halves provided, found by
/// type; `enablement` is the applied instance's enablement, which views
/// subscribe to.
#[derive(Clone)]
pub struct ShellContext {
    pub identity: &'static Identity,
    pub runtime: Handle,
    pub engine: Engine,
    pub store: Store,
    pub accounts: Accounts,
    pub http: HttpClient,
    pub paths: SpPaths,
    pub settings: Arc<dyn SettingsAccess>,
    pub handles: Handles,
    pub enablement: Enablement,
}

impl ShellContext {
    pub fn settings_for(&self, extension: &ExtensionId) -> ExtensionSettings {
        ExtensionSettings::new(extension.clone(), Arc::clone(&self.settings))
    }
}

impl fmt::Debug for ShellContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShellContext")
            .field("app", &self.identity.display_name())
            .field("handles", &self.handles)
            .field("enablement", &self.enablement)
            .finish_non_exhaustive()
    }
}
