//! Git-source orchestration failures retain their HTTP classification.

use super::AdminError;

impl From<systemprompt::system::managed::OrchestrationError> for AdminError {
    fn from(error: systemprompt::system::managed::OrchestrationError) -> Self {
        use systemprompt::system::managed::OrchestrationError;
        match error {
            OrchestrationError::Managed(error) => error.into(),
            OrchestrationError::Source(message) => Self::Conflict(message),
        }
    }
}
