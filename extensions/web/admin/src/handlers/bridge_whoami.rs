//! `GET /api/public/bridge/whoami` — the identity envelope the desktop bridge
//! shows on its account card.
//!
//! Adds the account's configured group/project membership and connected
//! provider state to core's identity response without changing login methods.
//!
//! The response deliberately reuses core's own key names for the fields it
//! knows (`user_id`, `email`, `display_name`, `provider`, `roles`) so they
//! land in typed slots rather than the passthrough map.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt::models::Config;
use systemprompt::models::auth::JwtAudience;
use systemprompt::oauth::validate_jwt_token;

use crate::error::{AdminError, AdminResult};
use crate::repositories;

use super::users::extract_token_from_headers;

// Why: a device-link token is minted `[bridge, mcp]` and a PAT-backed session
// is `api`, so all three are accepted; a token for none of them is not a
// bridge and is refused.
const BRIDGE_AUDIENCES: &[JwtAudience] = &[JwtAudience::Bridge, JwtAudience::Api, JwtAudience::Mcp];

// Why: flat on purpose. Core types the fields it knows and forwards the rest
// verbatim to the profile card, which humanises each key into a label — so a
// key here *is* a label, and a nested object would render as one unreadable
// row. `*_unix` is the suffix core reads as "format this as a date".
#[derive(Debug, Serialize)]
pub(crate) struct BridgeWhoamiResponse {
    connections: crate::services::connector_accounts::ConnectionSnapshot,
    user_id: UserId,
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    roles: Vec<String>,
    // Why: `None` for a local account; the bridge renders "Signed in with".
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,

    username: String,
    status: String,
    email_verified: bool,
    // Why: the groups carry entitlement and the projects carry work
    // attribution; both are shown so a person can see why a plugin is or is
    // not on their machine.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    groups: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    projects: Vec<String>,
    is_admin: bool,
    account_created_unix: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    identity_provider_issuer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    directory_subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    directory_linked_unix: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    directory_last_seen_unix: Option<i64>,
    // Why: mapped AD groups as of this user's most recent ADFS sign-in.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    directory_groups: Vec<String>,

    // Why: what the presented token itself says, as opposed to what the
    // database says. Shown so a stale or wrongly-scoped token is diagnosable
    // from the desktop instead of from the server logs.
    token_issuer: String,
    token_audiences: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    token_scope: Vec<String>,
    token_issued_unix: i64,
    token_expires_unix: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    login_session: Option<SessionId>,
}

const fn unix(ts: DateTime<Utc>) -> i64 {
    ts.timestamp()
}

pub(crate) async fn bridge_whoami_handler(
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
) -> AdminResult<Json<BridgeWhoamiResponse>> {
    let token = extract_token_from_headers(&headers)?;
    let jwt_issuer = Config::get()?.jwt_issuer.clone();
    let claims = validate_jwt_token(&token, &jwt_issuer, BRIDGE_AUDIENCES).map_err(|e| {
        tracing::warn!(error = %e, "bridge whoami: token rejected");
        AdminError::Unauthorized("Invalid or expired token".to_owned())
    })?;

    let user_id = UserId::new(claims.sub.clone());
    let envelope = repositories::users::queries::find_identity_envelope(&pool, &user_id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("User not found: {}", user_id.as_str())))?;

    // Why: a group read that fails must not fail the sign-in card. The rest of
    // the identity is still true and still worth showing.
    let adfs_groups = repositories::groups::members::list_source_ad_groups(&pool, &user_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "bridge whoami: AD group lookup failed"))
        .unwrap_or_default();

    let is_admin = envelope.roles.iter().any(|r| r == "admin");

    let connections = crate::services::connector_accounts::get_connections(&pool, &user_id).await?;
    Ok(Json(BridgeWhoamiResponse {
        connections,
        // Why: named for the mechanism that authenticated them, not for the
        // directory product — a rename of the IdP must not rewrite the string
        // this feeds into the audit trail.
        provider: envelope.idp_issuer.as_ref().map(|_| "adfs".to_owned()),
        user_id: envelope.user_id,
        email: envelope.email,
        display_name: envelope.display_name,
        roles: envelope.roles,
        username: envelope.username,
        status: envelope.status,
        email_verified: envelope.email_verified,
        groups: envelope.group_ids,
        projects: envelope.project_ids,
        is_admin,
        account_created_unix: unix(envelope.created_at),
        identity_provider_issuer: envelope.idp_issuer,
        directory_subject: envelope.external_sub,
        directory_linked_unix: envelope.linked_at.map(unix),
        directory_last_seen_unix: envelope.last_seen_at.map(unix),
        directory_groups: adfs_groups,
        token_issuer: claims.iss,
        token_audiences: claims.aud.iter().map(|a| a.as_str().to_owned()).collect(),
        token_scope: claims.scope.iter().map(ToString::to_string).collect(),
        token_issued_unix: claims.iat,
        token_expires_unix: claims.exp,
        login_session: claims.session_id,
    }))
}
