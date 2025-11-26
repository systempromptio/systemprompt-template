pub mod authentication;
pub mod authorization;
pub mod extraction;

pub use authentication::AuthenticationService;
pub use authorization::AuthorizationService;
pub use extraction::TokenExtractor;

// Legacy compatibility - provide the same interface as the old AuthService
#[derive(Debug, Copy, Clone)]
pub struct AuthService;

impl AuthService {
    pub fn extract_bearer_token(
        headers: &axum::http::HeaderMap,
    ) -> Result<String, axum::http::StatusCode> {
        TokenExtractor::extract_bearer_token(headers)
    }

    pub async fn authenticate(
        headers: &axum::http::HeaderMap,
        context: &systemprompt_core_system::AppContext,
    ) -> Result<systemprompt_models::AuthenticatedUser, axum::http::StatusCode> {
        AuthenticationService::authenticate(headers, context).await
    }

    pub async fn authorize_service_access(
        headers: &axum::http::HeaderMap,
        service_name: &str,
        context: &systemprompt_core_system::AppContext,
    ) -> Result<systemprompt_models::AuthenticatedUser, axum::http::StatusCode> {
        AuthorizationService::authorize_service_access(headers, service_name, context).await
    }

    pub async fn authorize_required_audience(
        headers: &axum::http::HeaderMap,
        required_audience: &str,
        context: &systemprompt_core_system::AppContext,
    ) -> Result<systemprompt_models::AuthenticatedUser, axum::http::StatusCode> {
        AuthorizationService::authorize_required_audience(headers, required_audience, context).await
    }
}
