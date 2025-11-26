use axum::http::Method;
use tower_http::cors::{AllowOrigin, CorsLayer};

#[derive(Debug, Clone, Copy)]
pub struct CorsMiddleware;

impl CorsMiddleware {
    pub fn build_layer() -> CorsLayer {
        let origins_str = std::env::var("CORS_ALLOWED_ORIGINS")
            .expect("CORS_ALLOWED_ORIGINS environment variable must be set");

        let origins: Vec<_> = origins_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.parse::<http::HeaderValue>()
                    .unwrap_or_else(|_| panic!("Invalid origin in CORS_ALLOWED_ORIGINS: {}", s))
            })
            .collect();

        if origins.is_empty() {
            panic!("CORS_ALLOWED_ORIGINS must contain at least one valid origin");
        }

        CorsLayer::new()
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
            ])
    }
}
