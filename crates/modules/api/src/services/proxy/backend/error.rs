use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("Service '{service}' not found in inventory")]
    ServiceNotFound { service: String },

    #[error("Service '{service}' is not running (status: {status})")]
    ServiceNotRunning { service: String, status: String },

    #[error("Failed to connect to {service} at {url}: {source}")]
    ConnectionFailed {
        service: String,
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("Request to {service} timed out")]
    Timeout { service: String },

    #[error("Invalid response from {service}: {reason}")]
    InvalidResponse { service: String, reason: String },

    #[error("Failed to build URL for {service}: {reason}")]
    UrlConstructionFailed { service: String, reason: String },

    #[error("Failed to extract request body: {source}")]
    BodyExtractionFailed {
        #[source]
        source: axum::Error,
    },

    #[error("Invalid HTTP method: {method}")]
    InvalidMethod { method: String },

    #[error("Database error when looking up service '{service}': {source}")]
    DatabaseError {
        service: String,
        #[source]
        source: anyhow::Error,
    },

    #[error("Authentication required for service '{service}'")]
    AuthenticationRequired { service: String },

    #[error("Access forbidden for service '{service}'")]
    Forbidden { service: String },

    #[error("Missing request context: {message}")]
    MissingContext { message: String },
}

impl ProxyError {
    pub const fn to_status_code(&self) -> StatusCode {
        match self {
            Self::ServiceNotFound { .. } => StatusCode::NOT_FOUND,
            Self::ServiceNotRunning { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::ConnectionFailed { .. } | Self::InvalidResponse { .. } => StatusCode::BAD_GATEWAY,
            Self::Timeout { .. } => StatusCode::GATEWAY_TIMEOUT,
            Self::UrlConstructionFailed { .. } | Self::DatabaseError { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            },
            Self::BodyExtractionFailed { .. } | Self::InvalidMethod { .. } => {
                StatusCode::BAD_REQUEST
            },
            Self::AuthenticationRequired { .. } | Self::MissingContext { .. } => {
                StatusCode::UNAUTHORIZED
            },
            Self::Forbidden { .. } => StatusCode::FORBIDDEN,
        }
    }
}

impl From<ProxyError> for StatusCode {
    fn from(error: ProxyError) -> Self {
        error.to_status_code()
    }
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        let status = self.to_status_code();
        let message = self.to_string();
        (status, message).into_response()
    }
}
