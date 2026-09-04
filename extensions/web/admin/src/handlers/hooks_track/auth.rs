//! Hook token validation for `/hooks/track`.
//!
//! Hook JWTs carry `aud=hook` and scope `hook:track`, distinct from the API and
//! plugin audiences.

use crate::error::{AdminError, AdminResult};
use axum::http::HeaderMap;
use systemprompt::identifiers::UserId;
use systemprompt::models::Config;
use systemprompt_security::HookTokenValidator;

pub(super) fn extract_and_validate_jwt(
    headers: &HeaderMap,
    request_plugin_id: Option<&str>,
) -> AdminResult<(UserId, String, String)> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AdminError::Unauthorized("Missing Authorization header".to_owned()))?;
    let jwt_issuer = Config::get()?.jwt_issuer.clone();
    let claims = HookTokenValidator::new(jwt_issuer)
        .validate_track(token, request_plugin_id)
        .map_err(AdminError::unauthenticated)?;
    Ok((
        claims.subject,
        claims.plugin_id.as_str().to_owned(),
        token.to_owned(),
    ))
}
