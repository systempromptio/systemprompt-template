//! Helpers shared across admin handlers.

use std::path::PathBuf;

use serde::Serialize;
use systemprompt::config::ProfileBootstrap;

use crate::error::AdminResult;

/// The one shape an admin API failure takes on the wire. Built only by
/// `AdminError::into_response`, so a status and its body cannot be chosen
/// independently of each other.
#[derive(Debug, Serialize)]
pub(crate) struct ErrorBody {
    pub error: String,
}

pub(crate) fn get_services_path() -> AdminResult<PathBuf> {
    Ok(PathBuf::from(&ProfileBootstrap::get()?.paths.services))
}

pub(crate) fn get_profile_path() -> AdminResult<PathBuf> {
    Ok(PathBuf::from(ProfileBootstrap::get_path()?))
}
