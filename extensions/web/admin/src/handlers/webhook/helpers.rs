//! Helpers shared by the webhook handlers.

use axum::http::HeaderMap;
use systemprompt::identifiers::UserId;
use systemprompt::models::auth::JwtAudience;

use crate::env::AdminEnv;
use crate::error::{AdminError, AdminResult};

// Why: accept a bearer token minted for any of the three audiences a Claude
// Code hook can be running under: the hook audience proper, a plugin token, or
// a plain API token for a caller driving the endpoint directly.
pub(super) const WEBHOOK_AUDIENCES: &[JwtAudience] = &[JwtAudience::Hook, JwtAudience::Api];

pub(super) async fn authenticate_webhook(
    env: &AdminEnv,
    headers: &HeaderMap,
) -> AdminResult<UserId> {
    let token = extract_bearer_token(headers)
        .ok_or_else(|| AdminError::Unauthorized("Missing Authorization header".to_owned()))?;
    let mut audiences = WEBHOOK_AUDIENCES.to_vec();
    audiences.push(JwtAudience::Resource("plugin".to_owned()));
    let (context, _) = crate::services::auth::verify_bearer(env, token, &audiences).await?;
    Ok(context.user_id)
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
