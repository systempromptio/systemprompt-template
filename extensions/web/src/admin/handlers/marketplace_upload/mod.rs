use std::path::PathBuf;

use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use systemprompt::models::auth::JwtAudience;
use systemprompt::models::{Config, ProfileBootstrap, SecretsBootstrap};
use systemprompt::oauth::validate_jwt_token;

mod api;
mod restore;
mod upload;

pub(crate) use api::{
    get_base_skill_content_handler, marketplace_all_versions_handler,
    marketplace_changelog_handler, marketplace_version_detail_handler,
    marketplace_versions_handler,
};
pub(crate) use restore::marketplace_restore_handler;
pub(crate) use upload::marketplace_upload_handler;

fn authenticate(headers: &HeaderMap, user_id: &str) -> Result<(), Box<Response>> {
    if let Some(token) = extract_bearer_token(headers) {
        let (jwt_secret, jwt_issuer) = get_jwt_config().map_err(|e| {
            tracing::error!(error = %e, "Failed to load JWT config");
            Box::new(
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Internal configuration error"})),
                )
                    .into_response(),
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
            Box::new(
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"error": "Invalid or expired token"})),
                )
                    .into_response(),
            )
        })?;

        if claims.sub != user_id {
            return Err(Box::new(
                (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"error": "Token does not match user ID"})),
                )
                    .into_response(),
            ));
        }

        return Ok(());
    }

    if let Ok((cookie_user_id, _, _)) = super::users::extract_user_from_cookie(headers) {
        let _ = cookie_user_id;
        return Ok(());
    }

    Err(Box::new(
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Missing Authorization Bearer token or session cookie"})),
        )
            .into_response(),
    ))
}

fn error_response(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({"error": message}))).into_response()
}

fn get_services_path() -> Result<PathBuf, Box<Response>> {
    let profile = ProfileBootstrap::get().map_err(|e| {
        tracing::error!(error = %e, "Failed to get profile bootstrap");
        Box::new(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load profile",
        ))
    })?;
    Ok(PathBuf::from(&profile.paths.services))
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

fn get_jwt_config() -> Result<(String, String), anyhow::Error> {
    let secret = SecretsBootstrap::jwt_secret()?.to_string();
    let issuer = Config::get()?.jwt_issuer.clone();
    Ok((secret, issuer))
}
