//! `POST /admin/auth/invite/accept` — the public half of the invite flow.
//!
//! Public by design: the invite token is the authorization. It exchanges a
//! valid token for a passkey setup token, provisioning the account as it goes,
//! so the invitee lands in the same enrolment path as self-registration.

use axum::Json;
use axum::extract::Extension;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use systemprompt::oauth::services::webauthn::{generate_setup_token, hash_token};

use crate::error::{AdminError, AdminResult};
use crate::handlers::salesforce_auth::SalesforceDeps;
use crate::repositories::users::{invites, passkey};

const SETUP_TOKEN_TTL_MINUTES: i64 = 10;

#[derive(Debug, Deserialize)]
pub(crate) struct AcceptInviteRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AcceptInviteResponse {
    pub email: String,
    pub setup_token: String,
    pub expires_in_seconds: i64,
}

// Why: public — the invite token is the authorization. It exchanges a valid
// token for a passkey setup token, provisioning the account as it goes; once
// the invite is marked accepted the invitee signs in with their passkey.
pub(crate) async fn accept_invite_handler(
    Extension(deps): Extension<SalesforceDeps>,
    Json(req): Json<AcceptInviteRequest>,
) -> AdminResult<Json<AcceptInviteResponse>> {
    let token = req.token.trim();
    if token.is_empty() {
        return Err(AdminError::BadRequest(
            "An invite token is required".to_owned(),
        ));
    }
    let token_hash = hash_token(token);
    let invite = invites::find_valid_invite_by_hash(&deps.write_pool, &token_hash)
        .await?
        .ok_or_else(|| {
            AdminError::NotFound(
                "This invite link is invalid, expired, or already used.".to_owned(),
            )
        })?;

    let user_id = invites::accept_invite_and_provision(&deps.write_pool, &invite).await?;
    crate::authz::organization::invalidate(&user_id).await;

    let (raw_token, setup_hash) = generate_setup_token();
    let expires_at = Utc::now() + Duration::minutes(SETUP_TOKEN_TTL_MINUTES);
    passkey::insert_setup_token(&deps.write_pool, &user_id, &setup_hash, expires_at).await?;

    tracing::info!(user_id = %user_id, email = %invite.email, "Invite accepted; setup token issued");

    Ok(Json(AcceptInviteResponse {
        email: invite.email,
        setup_token: raw_token,
        expires_in_seconds: SETUP_TOKEN_TTL_MINUTES * 60,
    }))
}
