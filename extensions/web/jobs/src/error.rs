use systemprompt::generator::PublishError;
use systemprompt::traits::ProviderError;
use systemprompt_web_shared::error::MarketplaceError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JobError {
    #[error("Job context missing required value: {0}")]
    MissingContext(&'static str),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Format error: {0}")]
    Format(#[from] std::fmt::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Marketplace error: {0}")]
    Marketplace(#[from] MarketplaceError),

    #[error("Publish error: {0}")]
    Publish(#[from] PublishError),

    #[error("Pipeline failed: {failed} sub-job(s) reported errors")]
    Pipeline { failed: u64 },

    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for JobError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<JobError> for ProviderError {
    fn from(err: JobError) -> Self {
        Self::Internal(err.to_string())
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
