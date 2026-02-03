use thiserror::Error;

use crate::identifiers::MemoryId;

#[derive(Error, Debug)]
pub enum SoulError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Memory not found: {0}")]
    MemoryNotFound(MemoryId),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),
}
