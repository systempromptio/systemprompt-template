//! HTTP handlers for the bridge control plane.

pub(crate) mod plugin_file;

use axum::body::Body;
use axum::http::{HeaderMap, Response, StatusCode};
use systemprompt::identifiers::UserId;
use systemprompt::models::Config;
use systemprompt::models::auth::JwtAudience;
use systemprompt::oauth::validate_jwt_token;

use crate::handlers::shared;

pub(super) fn validate_bridge_jwt(headers: &HeaderMap) -> Result<UserId, Box<Response<Body>>> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            shared::boxed_error_response(StatusCode::UNAUTHORIZED, "Missing Authorization header")
        })?;

    let jwt_issuer = Config::get()
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to load config");
            shared::boxed_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal configuration error",
            )
        })?
        .jwt_issuer
        .clone();

    let claims = validate_jwt_token(token, &jwt_issuer, &[JwtAudience::Bridge]).map_err(|err| {
        tracing::warn!(error = %err, "Bridge JWT validation failed");
        shared::boxed_error_response(StatusCode::UNAUTHORIZED, "Invalid or expired token")
    })?;

    Ok(UserId::new(&claims.sub))
}
