//! Bearer verification for the admin handlers.
//!
//! Every bearer is verified by core's open request authenticator
//! (`DynRequestAuthenticator`): the signature against the kernel keyring, the
//! issuer, the audiences the caller names, revocation, and the user and
//! session rows. The member never holds a verification key.

use crate::env::AdminEnv;

use axum::http::HeaderMap;
use systemprompt::identifiers::{JwtToken, UserId};
use systemprompt::models::auth::{JwtAudience, JwtUserContext};
use systemprompt::traits::AuthUser;

use crate::error::{AdminError, AdminResult};

pub(crate) async fn verify_bearer(
    env: &AdminEnv,
    token: &str,
    audiences: &[JwtAudience],
) -> AdminResult<(JwtUserContext, AuthUser)> {
    env.authenticator()
        .decode_for_audiences(&JwtToken::new(token), audiences)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "bearer rejected");
            AdminError::Unauthorized("Invalid or expired token".to_owned())
        })
}

pub(crate) fn bearer_from(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            v.strip_prefix("Bearer ")
                .or_else(|| v.strip_prefix("bearer "))
        })
        .map(str::trim)
        .filter(|t| !t.is_empty())
}

pub(crate) async fn validate_plugin_jwt(
    env: &AdminEnv,
    headers: &HeaderMap,
) -> AdminResult<UserId> {
    let token = bearer_from(headers)
        .ok_or_else(|| AdminError::Unauthorized("Missing Authorization header".to_owned()))?;
    let (context, _) =
        verify_bearer(env, token, &[JwtAudience::Resource("plugin".to_owned())]).await?;
    Ok(context.user_id)
}
