pub use systemprompt_traits::RepositoryError;

use crate::api::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("business logic error: {0}")]
    BusinessLogic(String),

    #[error("external service error: {0}")]
    External(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),
}

impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::Repository(e) => e.into(),
            ServiceError::Validation(msg) | ServiceError::BusinessLogic(msg) => {
                Self::bad_request(msg)
            },
            ServiceError::NotFound(msg) => Self::not_found(msg),
            ServiceError::External(msg) => {
                Self::internal_error(format!("External service error: {msg}"))
            },
            ServiceError::Conflict(msg) => Self::conflict(msg),
            ServiceError::Unauthorized(msg) => Self::unauthorized(msg),
            ServiceError::Forbidden(msg) => Self::forbidden(msg),
        }
    }
}

impl From<RepositoryError> for ApiError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound(msg) => Self::not_found(msg),
            RepositoryError::InvalidData(msg) | RepositoryError::ConstraintViolation(msg) => {
                Self::bad_request(msg)
            },
            RepositoryError::DatabaseError(e) => {
                Self::internal_error(format!("Database error: {e}"))
            },
            RepositoryError::Database(msg) => {
                Self::internal_error(format!("Database error: {msg}"))
            },
            RepositoryError::SerializationError(e) => {
                Self::internal_error(format!("Serialization error: {e}"))
            },
            RepositoryError::Serialization(msg) => {
                Self::internal_error(format!("Serialization error: {msg}"))
            },
            RepositoryError::GenericError(e) => Self::internal_error(format!("Error: {e}")),
        }
    }
}
