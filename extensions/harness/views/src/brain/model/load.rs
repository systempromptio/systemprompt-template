//! One read the views wait on: not asked yet, in flight, answered or failed.

use sp_brain::BrainError;

use super::failure::BrainFailure;

/// The state of one read.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Load<T> {
    #[default]
    Idle,
    Loading,
    Ready(T),
    Failed(BrainFailure),
}

impl<T> Load<T> {
    pub fn settle(result: Result<T, BrainError>, what: &'static str) -> Self {
        match result {
            Ok(value) => Self::Ready(value),
            Err(error) => {
                tracing::warn!(what, error = %error, "a brain read failed");
                Self::Failed(BrainFailure::of(&error))
            },
        }
    }

    pub const fn ready(&self) -> Option<&T> {
        match self {
            Self::Ready(value) => Some(value),
            _ => None,
        }
    }

    pub const fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    pub const fn failure(&self) -> Option<BrainFailure> {
        match self {
            Self::Failed(failure) => Some(*failure),
            _ => None,
        }
    }
}
