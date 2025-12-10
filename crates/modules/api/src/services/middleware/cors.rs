use axum::http::Method;
use thiserror::Error;
use tower_http::cors::{AllowOrigin, CorsLayer};

#[derive(Debug, Error)]
pub enum CorsError {
    #[error("CORS_ALLOWED_ORIGINS environment variable must be set")]
    MissingEnvVar,
    #[error("Invalid origin in CORS_ALLOWED_ORIGINS: {0}")]
    InvalidOrigin(String),
    #[error("CORS_ALLOWED_ORIGINS must contain at least one valid origin")]
    EmptyOrigins,
}

#[derive(Debug, Clone, Copy)]
pub struct CorsMiddleware;

impl CorsMiddleware {
    pub fn build_layer() -> Result<CorsLayer, CorsError> {
        let origins_str =
            std::env::var("CORS_ALLOWED_ORIGINS").map_err(|_| CorsError::MissingEnvVar)?;

        let mut origins = Vec::new();
        for origin in origins_str
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let header_value = origin
                .parse::<http::HeaderValue>()
                .map_err(|_| CorsError::InvalidOrigin(origin.to_string()))?;
            origins.push(header_value);
        }

        if origins.is_empty() {
            return Err(CorsError::EmptyOrigins);
        }

        Ok(CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_credentials(true)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                http::header::AUTHORIZATION,
                http::header::CONTENT_TYPE,
                http::header::ACCEPT,
                http::header::ORIGIN,
                http::header::ACCESS_CONTROL_REQUEST_METHOD,
                http::header::ACCESS_CONTROL_REQUEST_HEADERS,
                http::HeaderName::from_static("mcp-protocol-version"),
                http::HeaderName::from_static("x-context-id"),
                http::HeaderName::from_static("x-trace-id"),
                http::HeaderName::from_static("x-call-source"),
            ]))
    }
}
