//! Helpers shared by the webhook handlers.

use axum::http::HeaderMap;
use systemprompt::models::Config;
use systemprompt::models::auth::JwtAudience;

use crate::error::{AdminError, AdminResult};

#[derive(Debug, thiserror::Error)]
pub(super) enum JwtConfigError {
    #[error("Failed to load config: {0}")]
    Config(#[source] Box<dyn std::error::Error + Send + Sync>),
}

/// Accept a bearer token minted for any of the three audiences a Claude Code
/// hook can be running under: the hook audience proper, a plugin token, or a
/// plain API token for a caller driving the endpoint directly.
pub(super) fn authenticate_webhook(headers: &HeaderMap) -> AdminResult<()> {
    let token = extract_bearer_token(headers)
        .ok_or_else(|| AdminError::Unauthorized("Missing Authorization header".to_owned()))?;
    let jwt_issuer = get_jwt_issuer().map_err(AdminError::internal)?;
    systemprompt::oauth::validate_jwt_token(
        token,
        &jwt_issuer,
        &[
            JwtAudience::Resource("hook".to_owned()),
            JwtAudience::Resource("plugin".to_owned()),
            JwtAudience::Api,
        ],
    )?;
    Ok(())
}

pub(super) fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|v| {
            v.to_str()
                .inspect_err(|e| tracing::warn!(error = %e, "Non-ASCII authorization header"))
                .ok()
        })
        .and_then(|v| v.strip_prefix("Bearer "))
}

pub(super) fn get_jwt_issuer() -> Result<String, JwtConfigError> {
    let issuer = Config::get()
        .map_err(|e| JwtConfigError::Config(e.into()))?
        .jwt_issuer
        .clone();
    Ok(issuer)
}
