use axum::http::StatusCode;
use systemprompt::traits::ExtensionError;
use thiserror::Error;

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

impl ExtensionError for BlogError {
    fn code(&self) -> &'static str {
        match self {
            Self::DatabaseNotPostgres => "DATABASE_NOT_POSTGRES",
            Self::Database(_) => "DATABASE_ERROR",
            Self::ContentNotFound(_) => "CONTENT_NOT_FOUND",
            Self::LinkNotFound(_) => "LINK_NOT_FOUND",
            Self::InvalidRequest(_) => "INVALID_REQUEST",
            Self::Serialization(_) => "SERIALIZATION_ERROR",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Parse(_) => "PARSE_ERROR",
            Self::Io(_) => "IO_ERROR",
            Self::Yaml(_) => "YAML_ERROR",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::ContentNotFound(_) | Self::LinkNotFound(_) => StatusCode::NOT_FOUND,
            Self::InvalidRequest(_) | Self::Validation(_) | Self::Parse(_) => {
                StatusCode::BAD_REQUEST
            }
            Self::DatabaseNotPostgres
            | Self::Database(_)
            | Self::Serialization(_)
            | Self::Io(_)
            | Self::Yaml(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(self, Self::Database(_) | Self::Io(_))
    }
}
