//! The process-wide [`GovernanceEngine`] instance.
//!
//! Core now owns both the engine and the deployment decision this module used
//! to make — where the config lives (`<services>/governance/config.yaml` per
//! the profile) — so this is a thin delegation. It stays as a named seam
//! because the webhook is not the only enforcement point: the `/v1/messages`
//! gateway runs the same chain, and the rate limiter's buckets are
//! instance-scoped, so a second engine here would silently double every
//! budget.

use systemprompt_security::policy::GovernanceEngine;

pub(crate) fn engine()
-> Result<GovernanceEngine, systemprompt_security::policy::GovernanceEngineError> {
    let profile = systemprompt::config::ProfileBootstrap::get().map_err(|error| {
        systemprompt_security::policy::GovernanceEngineError::ConfigRejected {
            path: "profile".to_owned(),
            message: error.to_string(),
        }
    })?;
    GovernanceEngine::from_services_root(std::path::Path::new(&profile.paths.services))
}
