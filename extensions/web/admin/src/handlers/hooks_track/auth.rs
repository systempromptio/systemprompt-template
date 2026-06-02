use crate::handlers::shared;
use axum::{
    http::{HeaderMap, StatusCode},
    response::Response,
};
use systemprompt::identifiers::UserId;
use systemprompt::models::Config;
use systemprompt_security::HookTokenValidator;

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
    // Validate with the canonical hook-token validator (same path the gateway's
    // `/hooks/govern` endpoint uses): `aud` must contain `hook`, the `hook:track`
    // scope must be present, and the `plugin_id` claim must be set. Hook tokens
    // are minted with `audience=hook` + `scope=hook:govern hook:track`; the prior
    // hand-rolled check accepted `api`/`plugin` instead and rejected every real
    // hook token with InvalidAudience. `None` skips the request-vs-claim plugin_id
    // cross-check because this endpoint takes no plugin_id path/query binding.
    let claims = HookTokenValidator::new(jwt_issuer)
        .validate_track(token, None)
        .map_err(|e| {
            tracing::warn!(error = %e, "Hook tracking JWT validation failed");
            Box::new(shared::error_response(
                StatusCode::UNAUTHORIZED,
                "Invalid or expired token",
            ))
        })?;
    Ok((
        claims.subject,
        claims.plugin_id.as_str().to_string(),
        token.to_string(),
    ))
}
