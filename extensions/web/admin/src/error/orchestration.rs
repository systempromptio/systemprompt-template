//! Git-source sync failures retain their HTTP classification.

use super::AdminError;
use systemprompt::traits::GitSourceSyncError;

impl From<GitSourceSyncError> for AdminError {
    fn from(error: GitSourceSyncError) -> Self {
        match error {
            error @ (GitSourceSyncError::CredentialUnavailable
            | GitSourceSyncError::NotGitSource) => Self::Conflict(error.to_string()),
            error @ (GitSourceSyncError::Audit(_) | GitSourceSyncError::Failed(_)) => {
                Self::internal(error)
            },
        }
    }
}
