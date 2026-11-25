//! Preserve conflict and owner-scope failures at the HTTP boundary.

use super::AdminError;
use systemprompt::marketplace::managed::ManagedError;

impl From<ManagedError> for AdminError {
    fn from(error: ManagedError) -> Self {
        match error {
            ManagedError::Unavailable => Self::NotFound("Resource unavailable".to_owned()),
            ManagedError::Invalid { reason } => Self::Unprocessable(reason.to_owned()),
            ManagedError::Conflict { reason } => Self::Conflict(reason.to_owned()),
            other => Self::internal(other),
        }
    }
}

impl From<systemprompt::models::managed::RevisionBundleError> for AdminError {
    fn from(error: systemprompt::models::managed::RevisionBundleError) -> Self {
        ManagedError::from(error).into()
    }
}
