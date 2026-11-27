//! The pieces `Services::start` opens around the systemprompt side: the
//! accounts, named for this computer, and the engine, built over the governor
//! slot with the extensions' engine halves and the host's name for language
//! servers.

use std::ffi::OsString;
use std::sync::Arc;

use systemprompt_device::accounts::{Accounts, AccountsConfig};
use systemprompt_device::client::HttpClient;
use systemprompt_device::protocol::ids::InstallId;
use systemprompt_engine::config::Layout;
use systemprompt_engine::llm::HttpTransport;
use systemprompt_engine::mcp::ClientInfo;
use systemprompt_engine::run::extension::ExtensionSet;
use systemprompt_engine::run::{Engine, EngineOptions, Governor};
use systemprompt_engine::store::Store;
use systemprompt_harness_brand::Identity;
use systemprompt_models::bridge::host::HostKind;

use crate::app::browser::SystemBrowser;
use crate::app::governor::GovernorSlot;
use crate::app::paths::AppPaths;
use crate::app::services::StartupError;

pub(crate) struct EngineParts<'a> {
    pub(crate) identity: &'static Identity,
    pub(crate) paths: &'a AppPaths,
    pub(crate) layout: &'a Layout,
    pub(crate) store: &'a Store,
    pub(crate) governor: &'a Arc<GovernorSlot>,
    pub(crate) extensions: ExtensionSet,
    pub(crate) search_path: Option<OsString>,
    pub(crate) path_ext: Option<OsString>,
}

pub(crate) fn open_accounts(
    identity: &'static Identity,
    paths: &AppPaths,
    http: &HttpClient,
) -> Result<Accounts, StartupError> {
    Accounts::open(AccountsConfig {
        http: http.clone(),
        sealer: systemprompt_device::secure::platform_sealer(identity.app_id()),
        state_file: paths.accounts_file(),
        product: identity.product().to_owned(),
        install_id: InstallId::new(uuid::Uuid::new_v4().to_string()),
        browser: Arc::new(SystemBrowser),
        device_name: identity.device_label(&systemprompt_device::accounts::computer_name()),
    })
    .map_err(StartupError::Accounts)
}

pub(crate) fn open_engine(parts: EngineParts<'_>) -> Result<Engine, StartupError> {
    let EngineParts {
        identity,
        paths,
        layout,
        store,
        governor,
        extensions,
        search_path,
        path_ext,
    } = parts;
    let transport = HttpTransport::new().map_err(StartupError::ModelClient)?;
    let engine_governor: Arc<dyn Governor> = Arc::<GovernorSlot>::clone(governor);
    let mut client_info = ClientInfo::new(HostKind::Harness.as_str(), identity.version());
    client_info.title = Some(identity.display_name().to_owned());
    Engine::new(EngineOptions {
        paths: paths.engine_paths(),
        layout: layout.clone(),
        store: store.clone(),
        governor: engine_governor,
        transport: Arc::new(transport),
        client_info,
        search_path,
        path_ext,
        extensions,
        lsp_client_name: Some(HostKind::Harness.as_str().to_owned()),
    })
    .map_err(StartupError::Engine)
}
