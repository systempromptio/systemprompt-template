use std::path::PathBuf;

use axum::{
    http::{HeaderMap, StatusCode},
    response::Response,
};
use systemprompt::models::auth::JwtAudience;
use systemprompt::models::{Config, SecretsBootstrap};
use systemprompt::oauth::validate_jwt_token;

use super::shared;

mod api;
mod restore;
mod upload;

pub use api::{
    get_base_skill_content_handler, marketplace_all_versions_handler,
    marketplace_changelog_handler, marketplace_version_detail_handler,
    marketplace_versions_handler,
};
pub use restore::marketplace_restore_handler;
pub use upload::marketplace_upload_handler;

fn authenticate(headers: &HeaderMap, user_id: &str) -> Result<(), Box<Response>> {
    if let Some(token) = extract_bearer_token(headers) {
        let (jwt_secret, jwt_issuer) = get_jwt_config().map_err(|e| {
            tracing::error!(error = %e, "Failed to load JWT config");
            shared::boxed_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal configuration error",
            )
        })?;

        let claims = validate_jwt_token(
            token,
            &jwt_secret,
            &jwt_issuer,
            &[JwtAudience::Resource("plugin".to_string())],
        )
        .map_err(|e| {
            tracing::warn!(error = %e, "Marketplace upload JWT validation failed");
            shared::boxed_error_response(StatusCode::UNAUTHORIZED, "Invalid or expired token")
        })?;

        if claims.sub != user_id {
            return Err(shared::boxed_error_response(
                StatusCode::FORBIDDEN,
                "Token does not match user ID",
            ));
        }

        return Ok(());
    }

    if super::users::extract_user_from_cookie(headers).is_ok() {
        return Ok(());
    }

    Err(shared::boxed_error_response(
        StatusCode::UNAUTHORIZED,
        "Missing Authorization Bearer token or session cookie",
    ))
}

fn error_response(status: StatusCode, message: &str) -> Response {
    shared::error_response(status, message)
}

fn get_services_path() -> Result<PathBuf, Box<Response>> {
    shared::get_services_path()
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
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

fn get_jwt_config() -> Result<(String, String), crate::error::MarketplaceError> {
    let secret = SecretsBootstrap::jwt_secret()
        .map_err(|e| {
            crate::error::MarketplaceError::Internal(format!("Failed to load JWT secret: {e}"))
        })?
        .to_string();
    let issuer = Config::get()
        .map_err(|e| {
            crate::error::MarketplaceError::Internal(format!("Failed to load config: {e}"))
        })?
        .jwt_issuer
        .clone();
    Ok((secret, issuer))
}
