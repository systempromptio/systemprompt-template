//! Preserve conflict and owner-scope failures at the HTTP boundary.

use super::AdminError;
use systemprompt::marketplace::managed::ManagedError;

impl From<ManagedError> for AdminError {
    fn from(error: ManagedError) -> Self {
        match error {
            ManagedError::Unavailable => Self::NotFound("Resource unavailable".to_owned()),
            ManagedError::Invalid(message) => Self::Unprocessable(message),
            ManagedError::Conflict(message) => Self::Conflict(message),
            other => Self::internal(other),
        }
    }
}
