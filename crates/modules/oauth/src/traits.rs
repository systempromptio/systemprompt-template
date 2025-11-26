use async_trait::async_trait;
use systemprompt_models::auth::{AuthError, AuthenticatedUser};

#[async_trait]
pub trait TokenValidator: Send + Sync {
    async fn validate_token(&self, token: &str) -> Result<AuthenticatedUser, AuthError>;
}

pub fn extract_bearer_token(headers: &axum::http::HeaderMap) -> Result<String, AuthError> {
    headers
        .get("authorization")
        .ok_or(AuthError::AuthenticationFailed {
            message: "Authorization header missing".to_string(),
        })?
        .to_str()
        .map_err(|_| AuthError::InvalidTokenFormat)?
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidTokenFormat)
        .map(ToString::to_string)
}

pub fn extract_cookie_token(headers: &axum::http::HeaderMap) -> Result<String, AuthError> {
    headers
        .get("cookie")
        .ok_or(AuthError::AuthenticationFailed {
            message: "Cookie header missing".to_string(),
        })?
        .to_str()
        .map_err(|_| AuthError::InvalidTokenFormat)?
        .split(';')
        .find_map(|cookie| {
            let cookie = cookie.trim();
            if cookie.starts_with("access_token=") {
                cookie
                    .strip_prefix("access_token=")
                    .map(ToString::to_string)
            } else {
                None
            }
        })
        .ok_or(AuthError::AuthenticationFailed {
            message: "Access token not found in cookies".to_string(),
        })
}
