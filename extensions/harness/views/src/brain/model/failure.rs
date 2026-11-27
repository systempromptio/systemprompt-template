//! How a failed brain call is shown: the error reduced to what the person can
//! act on. The error itself is logged where it happens.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use sp_brain::BrainError;
use sp_client::ClientError;

/// What went wrong with a brain call, as a view tells the person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrainFailure {
    NoInstance,
    Unconfirmed,
    Disabled,
    NotFound,
    Forbidden,
    Tombstoned,
    Conflict,
    Invalid,
    SignedOut,
    RateLimited,
    Unreachable,
    Unexpected,
}

impl BrainFailure {
    pub const fn of(error: &BrainError) -> Self {
        match error {
            BrainError::NoInstance => Self::NoInstance,
            BrainError::Unconfirmed => Self::Unconfirmed,
            BrainError::Disabled => Self::Disabled,
            BrainError::NotFound { .. } => Self::NotFound,
            BrainError::Forbidden { .. } => Self::Forbidden,
            BrainError::Tombstoned { .. } => Self::Tombstoned,
            BrainError::Conflict { .. } => Self::Conflict,
            BrainError::Invalid { .. } | BrainError::InvalidInput { .. } => Self::Invalid,
            BrainError::Client(error) => Self::of_client(error),
        }
    }

    const fn of_client(error: &ClientError) -> Self {
        match error {
            ClientError::Unauthorized { .. }
            | ClientError::SessionMismatch { .. }
            | ClientError::Credential(_) => Self::SignedOut,
            ClientError::RateLimited { .. } => Self::RateLimited,
            ClientError::Unreachable { .. } => Self::Unreachable,
            _ => Self::Unexpected,
        }
    }

    pub const fn is_retryable(self) -> bool {
        matches!(
            self,
            Self::Unconfirmed | Self::RateLimited | Self::Unreachable | Self::Unexpected
        )
    }
}
