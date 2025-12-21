//! Blog extension error types.

use thiserror::Error;

/// Errors that can occur in the blog extension.
#[derive(Error, Debug)]
pub enum BlogError {
    #[error("Database must be PostgreSQL")]
    DatabaseNotPostgres,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Content not found: {0}")]
    ContentNotFound(String),

    #[error("Link not found: {0}")]
    LinkNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}
