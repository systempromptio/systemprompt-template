use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
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
    pub fn to_status_code(&self) -> StatusCode {
        match self {
            Self::ServiceNotFound { .. } => StatusCode::NOT_FOUND,
            Self::ServiceNotRunning { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::ConnectionFailed { .. } => StatusCode::BAD_GATEWAY,
            Self::Timeout { .. } => StatusCode::GATEWAY_TIMEOUT,
            Self::InvalidResponse { .. } => StatusCode::BAD_GATEWAY,
            Self::UrlConstructionFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BodyExtractionFailed { .. } => StatusCode::BAD_REQUEST,
            Self::InvalidMethod { .. } => StatusCode::BAD_REQUEST,
            Self::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::AuthenticationRequired { .. } => StatusCode::UNAUTHORIZED,
            Self::Forbidden { .. } => StatusCode::FORBIDDEN,
            Self::MissingContext { .. } => StatusCode::UNAUTHORIZED,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_not_found_returns_404() {
        let error = ProxyError::ServiceNotFound {
            service: "test-service".to_string(),
        };
        assert_eq!(error.to_status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn service_not_running_returns_503() {
        let error = ProxyError::ServiceNotRunning {
            service: "test-service".to_string(),
            status: "stopped".to_string(),
        };
        assert_eq!(error.to_status_code(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn body_extraction_failed_returns_400() {
        let axum_error = axum::Error::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "test error",
        ));
        let error = ProxyError::BodyExtractionFailed { source: axum_error };
        assert_eq!(error.to_status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn auth_required_returns_401() {
        let error = ProxyError::AuthenticationRequired {
            service: "test-service".to_string(),
        };
        assert_eq!(error.to_status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn forbidden_returns_403() {
        let error = ProxyError::Forbidden {
            service: "test-service".to_string(),
        };
        assert_eq!(error.to_status_code(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn error_messages_include_service_context() {
        let error = ProxyError::ServiceNotFound {
            service: "my-service".to_string(),
        };
        let message = error.to_string();
        assert!(message.contains("my-service"));
    }
}
