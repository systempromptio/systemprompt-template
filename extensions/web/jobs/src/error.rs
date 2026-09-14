//! Error type shared by every job in this crate.

use systemprompt::generator::PublishError;
use systemprompt::traits::ProviderError;
use systemprompt_web_admin::repositories::secrets::secret_crypto::SecretCryptoError;
use systemprompt_web_shared::error::{InfraError, MarketplaceError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JobError {
    #[error("Job context missing required value: {0}")]
    MissingContext(&'static str),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Configuration error: {0}")]
    CoreConfig(#[from] systemprompt::models::errors::ConfigError),

    #[error("Profile error: {0}")]
    Profile(#[from] systemprompt::config::ProfileBootstrapError),

    #[error("Services config error: {0}")]
    Services(#[from] systemprompt::loader::ConfigLoadError),

    #[error("Content error: {0}")]
    Content(#[from] systemprompt::content::ContentError),

    #[error("Secret crypto error: {0}")]
    SecretCrypto(#[from] SecretCryptoError),

    #[error("Format error: {0}")]
    Format(#[from] std::fmt::Error),

    #[error(transparent)]
    Infra(#[from] InfraError),

    #[error("Marketplace error: {0}")]
    Marketplace(#[from] MarketplaceError),

    #[error("Publish error: {0}")]
    Publish(#[from] PublishError),

    #[error("Pipeline failed: {failed} sub-job(s) reported errors")]
    Pipeline { failed: u64 },

    #[error("{0}")]
    Other(String),
}

impl From<JobError> for ProviderError {
    fn from(err: JobError) -> Self {
        Self::Internal(err.to_string())
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
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}
