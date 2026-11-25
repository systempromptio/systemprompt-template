//! Error type shared by every admin-plane job.

use systemprompt::traits::ProviderError;
use crate::repositories::secrets::secret_crypto::SecretCryptoError;
use systemprompt_web_shared::error::{InfraError, MarketplaceError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JobError {
    #[error(transparent)]
    Identifier(#[from] systemprompt::identifiers::error::IdValidationError),
    #[error(transparent)]
    Ai(#[from] systemprompt::ai::error::AiError),
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error("Job context missing required value: {0}")]
    MissingContext(#[from] systemprompt::traits::MissingDependency),

    #[error("Secret crypto error: {0}")]
    SecretCrypto(#[from] SecretCryptoError),

    #[error(transparent)]
    Infra(#[from] InfraError),

    #[error("Marketplace error: {0}")]
    Marketplace(#[from] MarketplaceError),

    #[error("AI inference failed: {0}")]
    Inference(#[from] systemprompt::models::errors::AiInferenceError),

    #[error("{0}")]
    Other(String),
}

impl From<JobError> for ProviderError {
    fn from(err: JobError) -> Self {
        Self::Internal(Box::new(err))
    }
}

impl From<sqlx::Error> for JobError {
    fn from(e: sqlx::Error) -> Self {
        Self::Infra(InfraError::from(e))
    }
}

impl From<std::io::Error> for JobError {
    fn from(e: std::io::Error) -> Self {
        Self::Infra(InfraError::from(e))
    }
}

impl From<serde_yaml::Error> for JobError {
    fn from(e: serde_yaml::Error) -> Self {
        Self::Infra(InfraError::from(e))
    }
}

impl From<serde_json::Error> for JobError {
    fn from(e: serde_json::Error) -> Self {
        Self::Infra(InfraError::from(e))
    }
}

impl JobError {
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}
