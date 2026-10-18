//! The process-wide [`GovernanceEngine`] instance.
//!
//! Core now owns both the engine and the deployment decision this module used
//! to make — where the config lives (`<services>/governance/config.yaml` per
//! the profile) — so this is a thin delegation. It stays as a named seam
//! because the webhook is not the only enforcement point: the `/v1/messages`
//! gateway runs the same chain. The rate limiter's windows are counted in
//! Postgres and shared by every engine, so this one is handed the same
//! cluster-wide window repository.

use systemprompt_security::policy::{GovernanceEngine, RateWindowRepository};

pub(crate) fn engine(
    rate_windows: &RateWindowRepository,
) -> crate::error::AdminResult<GovernanceEngine> {
    let profile = systemprompt::config::ProfileBootstrap::get()?;
    GovernanceEngine::from_services_root(std::path::Path::new(&profile.paths.services))
        .map(|engine| engine.with_rate_windows(rate_windows.clone()))
        .map_err(crate::error::AdminError::internal)
}
