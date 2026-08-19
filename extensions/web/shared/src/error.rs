//! Error types shared across the web extension crates.
//!
//! [`InfraError`] carries the infrastructure failures (database, IO,
//! serialization) common to every domain enum, so each domain enum holds a
//! single transparent `Infra` variant instead of a private copy of the same
//! four. `From` impls for the underlying error types are provided on the
//! domain enums directly, so `?` at call sites is unaffected.

use axum::http::StatusCode;
use systemprompt::traits::ExtensionError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InfraError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl InfraError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Database(_) => "DATABASE_ERROR",
            Self::Io(_) => "IO_ERROR",
            Self::Yaml(_) => "YAML_ERROR",
            Self::Json(_) => "JSON_ERROR",
        }
    }

    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(self, Self::Database(_) | Self::Io(_))
    }
}

macro_rules! infra_from {
    ($enum_:ident: $($source:ty),+) => {
        $(impl From<$source> for $enum_ {
            fn from(e: $source) -> Self {
                Self::Infra(InfraError::from(e))
            }
        })+
    };
}

#[derive(Error, Debug)]
pub enum MarketplaceError {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    /// The request was well-formed but conflicts with the current state — a
    /// seat limit already reached, a slug already taken. Distinct from
    /// [`Self::BadRequest`] because the caller has nothing to fix in the
    /// request itself, and a UI should say "your plan is full", not "invalid".
    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    /// A `services/` resource file failed to parse or validate, with the
    /// offending path (and, where useful, the entry within it) kept alongside
    /// the typed cause.
    #[error("{path}: {source}")]
    ConfigFile {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Profile error: {0}")]
    Profile(#[from] systemprompt::config::ProfileBootstrapError),

    #[error(transparent)]
    Infra(#[from] InfraError),
}

impl MarketplaceError {
    pub fn config_file(
        path: impl Into<String>,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::ConfigFile {
            path: path.into(),
            source: source.into(),
        }
    }
}

infra_from!(MarketplaceError: sqlx::Error, std::io::Error, serde_yaml::Error, serde_json::Error);

impl ExtensionError for MarketplaceError {
    fn code(&self) -> &'static str {
        match self {
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::Crypto(_) => "CRYPTO_ERROR",
            Self::ConfigFile { .. } => "CONFIG_FILE_ERROR",
            Self::Profile(_) => "PROFILE_ERROR",
            Self::Infra(e) => e.code(),
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Internal(_)
            | Self::Crypto(_)
            | Self::ConfigFile { .. }
            | Self::Profile(_)
            | Self::Infra(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(self, Self::Infra(e) if e.is_retryable())
    }
}

#[derive(Error, Debug)]
pub enum BlogError {
    #[error("Database must be PostgreSQL")]
    DatabaseNotPostgres,

    #[error("Content not found: {0}")]
    ContentNotFound(String),

    #[error("Link not found: {0}")]
    LinkNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error(transparent)]
    Infra(#[from] InfraError),
}

infra_from!(BlogError: sqlx::Error, std::io::Error, serde_yaml::Error, serde_json::Error);

impl ExtensionError for BlogError {
    fn code(&self) -> &'static str {
        match self {
            Self::DatabaseNotPostgres => "DATABASE_NOT_POSTGRES",
            Self::ContentNotFound(_) => "CONTENT_NOT_FOUND",
            Self::LinkNotFound(_) => "LINK_NOT_FOUND",
            Self::InvalidRequest(_) => "INVALID_REQUEST",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Parse(_) => "PARSE_ERROR",
            Self::Infra(e) => e.code(),
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::ContentNotFound(_) | Self::LinkNotFound(_) => StatusCode::NOT_FOUND,
            Self::InvalidRequest(_) | Self::Validation(_) | Self::Parse(_) => {
                StatusCode::BAD_REQUEST
            },
            Self::DatabaseNotPostgres | Self::Infra(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(self, Self::Infra(e) if e.is_retryable())
    }
}
