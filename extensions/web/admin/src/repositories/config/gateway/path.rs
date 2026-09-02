//! Where the gateway routes live on disk.
//!
//! Core 0.44 moved `gateway:` out of the profile and into the services tree.
//! The admin editor writes that file directly, following core's own
//! `admin config gateway`, which edits the services files and never the
//! profile: a `gateway:` key written back into `profile.yaml` is rejected at
//! the next boot with `ProfileError::MovedToServices`.

use std::path::PathBuf;

use systemprompt::config::ProfileBootstrap;
use systemprompt::loader::ServicesBootstrap;
use systemprompt_web_shared::error::MarketplaceError;

pub fn gateway_config_path() -> Result<PathBuf, MarketplaceError> {
    // Why: `settings.services_path` is the loaded tree's own statement of where
    // it lives, and it is what a services tree loaded from somewhere other than
    // the profile's `paths.services` sets. Preferring it keeps the file the
    // editor writes and the tree the gateway dispatches from as one place; the
    // profile path is the answer whenever the tree does not name itself.
    let from_services = ServicesBootstrap::get()
        .ok()
        .and_then(|services| services.settings.services_path.clone());
    if let Some(root) = from_services {
        return Ok(PathBuf::from(root).join("ai").join("gateway.yaml"));
    }
    let profile = ProfileBootstrap::get()
        // Why: lint-ok: error-adapt — ProfileError is core's variant-less config error.
        .map_err(|e| MarketplaceError::Internal(format!("profile is not initialised: {e}")))?;
    Ok(PathBuf::from(&profile.paths.services)
        .join("ai")
        .join("gateway.yaml"))
}
