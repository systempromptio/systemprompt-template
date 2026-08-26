//! Error type for the knowledge-bank MCP server.

use axum::http::StatusCode;
use systemprompt::traits::ExtensionError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KnowledgeBankError {
    #[error("Knowledge backend not configured: {0}")]
    NotConfigured(String),

    #[error("Unknown knowledge source: {0}")]
    UnknownSource(String),

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl ExtensionError for KnowledgeBankError {
    fn code(&self) -> &'static str {
        match self {
            Self::NotConfigured(_) => "BACKEND_NOT_CONFIGURED",
            Self::UnknownSource(_) => "UNKNOWN_SOURCE",
            Self::Backend(_) => "BACKEND_ERROR",
            Self::Serialization(_) => "SERIALIZATION_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::UnknownSource(_) => StatusCode::NOT_FOUND,
            Self::NotConfigured(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Backend(_) | Self::Serialization(_) | Self::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            },
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(self, Self::Backend(_))
    }
}
