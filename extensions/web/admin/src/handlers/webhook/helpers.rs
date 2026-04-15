use axum::http::HeaderMap;
use systemprompt::models::{Config, SecretsBootstrap};

#[derive(Debug, thiserror::Error)]
pub(super) enum JwtConfigError {
    #[error("Failed to load JWT secret: {0}")]
    Secret(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to load config: {0}")]
    Config(#[source] Box<dyn std::error::Error + Send + Sync>),
}

pub(super) fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|v| {
            v.to_str()
                .map_err(|e| {
                    tracing::warn!(error = %e, "Non-ASCII authorization header");
                })
                .ok()
        })
        .and_then(|v| v.strip_prefix("Bearer "))
}

pub(super) fn get_jwt_config() -> Result<(String, String), JwtConfigError> {
    let secret = SecretsBootstrap::jwt_secret()
        .map_err(|e| JwtConfigError::Secret(e.into()))?
        .to_string();
    let issuer = Config::get()
        .map_err(|e| JwtConfigError::Config(e.into()))?
        .jwt_issuer
        .clone();
    Ok((secret, issuer))
}
