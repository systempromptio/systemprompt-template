use thiserror::Error;

#[derive(Error, Debug)]
pub enum DocsError {
    #[error("Database not available in context")]
    NoDatabaseInContext,

    #[error("Database pool not initialized")]
    PoolNotInitialized,

    #[error("Content not found: {0}")]
    ContentNotFound(String),

    #[error("Content item required for docs page")]
    ContentItemRequired,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
