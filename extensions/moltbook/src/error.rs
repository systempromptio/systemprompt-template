use axum::http::StatusCode;
use systemprompt::traits::ExtensionError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MoltbookError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Post not found: {0}")]
    PostNotFound(String),

    #[error("Comment not found: {0}")]
    CommentNotFound(String),

    #[error("Submolt not found: {0}")]
    SubmoltNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Authorization error: {0}")]
    Unauthorized(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Prompt injection detected: {0}")]
    PromptInjection(String),

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Configuration error: {0}")]
    Configuration(String),
}

impl ExtensionError for MoltbookError {
    fn code(&self) -> &'static str {
        match self {
            Self::Database(_) => "DATABASE_ERROR",
            Self::Http(_) => "HTTP_ERROR",
            Self::AgentNotFound(_) => "AGENT_NOT_FOUND",
            Self::PostNotFound(_) => "POST_NOT_FOUND",
            Self::CommentNotFound(_) => "COMMENT_NOT_FOUND",
            Self::SubmoltNotFound(_) => "SUBMOLT_NOT_FOUND",
            Self::InvalidRequest(_) => "INVALID_REQUEST",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Serialization(_) => "SERIALIZATION_ERROR",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::RateLimitExceeded(_) => "RATE_LIMIT_EXCEEDED",
            Self::PromptInjection(_) => "PROMPT_INJECTION_DETECTED",
            Self::ApiError { .. } => "API_ERROR",
            Self::Configuration(_) => "CONFIGURATION_ERROR",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::AgentNotFound(_)
            | Self::PostNotFound(_)
            | Self::CommentNotFound(_)
            | Self::SubmoltNotFound(_) => StatusCode::NOT_FOUND,
            Self::InvalidRequest(_) | Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::RateLimitExceeded(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::PromptInjection(_) => StatusCode::FORBIDDEN,
            Self::Database(_) | Self::Http(_) | Self::Serialization(_) | Self::ApiError { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Database(_) | Self::Http(_) | Self::RateLimitExceeded(_)
        )
    }
}
