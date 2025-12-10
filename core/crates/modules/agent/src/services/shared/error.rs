use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentServiceError {
    #[error("database operation failed: {0}")]
    Database(String),

    #[error("repository operation failed: {0}")]
    Repository(String),

    #[error("network request failed: {0}")]
    Network(String),

    #[error("authentication failed: {0}")]
    Authentication(String),

    #[error("authorization failed for resource: {0}")]
    Authorization(String),

    #[error("validation failed: {0}: {1}")]
    Validation(String, String),

    #[error("resource not found: {0}")]
    NotFound(String),

    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("operation timed out after {0}ms")]
    Timeout(u64),

    #[error("configuration error: {0}: {1}")]
    Configuration(String, String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("logging error: {0}")]
    Logging(String),

    #[error("capacity exceeded: {0}")]
    Capacity(String),
}

impl From<sqlx::Error> for AgentServiceError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.to_string())
    }
}

impl From<crate::repository::RepositoryError> for AgentServiceError {
    fn from(err: crate::repository::RepositoryError) -> Self {
        Self::Repository(err.to_string())
    }
}

impl From<reqwest::Error> for AgentServiceError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(
            err.url()
                .map(|u| u.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        )
    }
}

impl From<anyhow::Error> for AgentServiceError {
    fn from(err: anyhow::Error) -> Self {
        Self::Configuration("unknown".to_string(), err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AgentServiceError>;
