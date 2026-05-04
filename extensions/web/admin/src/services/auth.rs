use axum::http::HeaderMap;
use systemprompt::config::SecretsBootstrap;
use systemprompt::models::auth::JwtAudience;
use systemprompt::models::Config;
use systemprompt::oauth::validate_jwt_token;

use crate::error::{AdminError, AdminResult};

pub fn validate_plugin_jwt(headers: &HeaderMap) -> AdminResult<String> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AdminError::Unauthorized("Missing Authorization header".to_string()))?;

    let jwt_secret = SecretsBootstrap::jwt_secret().map_err(|e| {
        tracing::error!(error = %e, "Failed to load JWT secret");
        AdminError::internal(e)
    })?;

    let jwt_issuer = Config::get()
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to load config");
            AdminError::internal(e)
        })?
        .jwt_issuer
        .clone();

    let claims = validate_jwt_token(
        token,
        jwt_secret,
        &jwt_issuer,
        &[JwtAudience::Resource("plugin".to_string())],
    )
    .map_err(|e| {
        tracing::warn!(error = %e, "Plugin JWT validation failed");
        AdminError::Unauthorized("Invalid or expired token".to_string())
    })?;

    Ok(claims.sub)
}
