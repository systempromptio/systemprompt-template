use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoworkRepoError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Validation error: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, CoworkRepoError>;
