use crate::handlers::shared;
use axum::{
    http::{HeaderMap, StatusCode},
    response::Response,
};
use systemprompt::identifiers::UserId;
use systemprompt::models::auth::JwtAudience;
use systemprompt::models::{Config, SecretsBootstrap};
use systemprompt::oauth::validate_jwt_token;

pub fn extract_and_validate_jwt(
    headers: &HeaderMap,
) -> Result<(UserId, String, String), Box<Response>> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            Box::new(shared::error_response(
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header",
            ))
        })?;
    let jwt_secret = SecretsBootstrap::jwt_secret().map_err(|e| {
        tracing::error!(error = %e, "Failed to load JWT secret");
        Box::new(shared::error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal configuration error",
        ))
    })?;
    let jwt_issuer = Config::get()
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to load config");
            Box::new(shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal configuration error",
            ))
        })?
        .jwt_issuer
        .clone();
    let claims = validate_jwt_token(
        token,
        jwt_secret,
        &jwt_issuer,
        &[
            JwtAudience::Resource("plugin".to_string()),
            JwtAudience::Api,
        ],
    )
    .map_err(|e| {
        tracing::warn!(error = %e, "Hook tracking JWT validation failed");
        Box::new(shared::error_response(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired token",
        ))
    })?;
    let plugin_id = claims
        .session_id
        .as_deref()
        .and_then(|s| s.strip_prefix("plugin_"))
        .unwrap_or("")
        .to_string();
    Ok((
        UserId::new(claims.sub.clone()),
        plugin_id,
        token.to_string(),
    ))
}
