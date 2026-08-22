//! `POST /admin/auth/passkey/register` — provision a user for an allow-listed
//! email and mint a `webauthn_setup_tokens` row for passkey enrolment.

use axum::{Extension, Json};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use systemprompt::identifiers::UserId;
use systemprompt::oauth::services::webauthn::generate_setup_token;
use systemprompt_web_shared::error::MarketplaceError;

use crate::error::{AdminError, AdminResult};
use crate::handlers::salesforce_auth::SalesforceDeps;
use crate::repositories::users::passkey;

const SETUP_TOKEN_TTL_MINUTES: i64 = 10;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterRequest {
    email: String,
    display_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct RegisterResponse {
    setup_token: String,
    expires_in_seconds: i64,
}

pub(crate) async fn passkey_register(
    Extension(deps): Extension<SalesforceDeps>,
    Json(req): Json<RegisterRequest>,
) -> AdminResult<Json<RegisterResponse>> {
    let email = req.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return Err(AdminError::BadRequest(
            "A valid email address is required".to_owned(),
        ));
    }
    // Why: this precedes the domain check because with closed enrolment the
    // domain is not the question. Answering "your domain is not eligible" to
    // an out-of-domain address would also confirm which domains *are*, so both
    // cases get the same refusal.
    if !deps.config.allow_self_registration {
        return Err(AdminError::Forbidden(
            "This platform does not accept self-registration. \
             Ask an administrator for an invite link."
                .to_owned(),
        ));
    }
    if !deps.config.email_allowed(&email) {
        return Err(AdminError::Forbidden(
            "This email domain is not eligible for self-registration. \
             Use your work email or contact the platform team."
                .to_owned(),
        ));
    }

    let display_name = req
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .unwrap_or(&email)
        .to_owned();

    let user_id = resolve_user(&deps, &email, &display_name).await?;

    let (raw_token, token_hash) = generate_setup_token();
    let expires_at = Utc::now() + Duration::minutes(SETUP_TOKEN_TTL_MINUTES);
    passkey::insert_setup_token(&deps.write_pool, &user_id, &token_hash, expires_at).await?;

    tracing::info!(user_id = %user_id, email, "Passkey self-registration setup token issued");

    Ok(Json(RegisterResponse {
        setup_token: raw_token,
        expires_in_seconds: SETUP_TOKEN_TTL_MINUTES * 60,
    }))
}

// Why: an existing account without a credential (e.g. SSO-provisioned) may
// enrol a passkey through this door; an account that already has one must sign
// in with it — handing out enrolment tokens for it would let anyone who knows
// an email add their own credential to that account.
async fn resolve_user(
    deps: &SalesforceDeps,
    email: &str,
    display_name: &str,
) -> AdminResult<UserId> {
    if let Some(existing) = passkey::find_user_by_email(&deps.write_pool, email).await? {
        if existing.has_passkey {
            return Err(AdminError::Conflict(
                "An account with this email already exists — sign in with your passkey".to_owned(),
            ));
        }
        return Ok(existing.id);
    }

    passkey::insert_passkey_user(&deps.write_pool, email, display_name)
        .await
        .map_err(|e| match e {
            MarketplaceError::Conflict(_) => AdminError::Conflict(
                "Your organization has no seats left. Contact your administrator.".to_owned(),
            ),
            other => AdminError::Marketplace(other),
        })
}
