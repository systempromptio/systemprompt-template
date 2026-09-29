//! HTTP handlers for registered device management.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminResult};
use crate::repositories;
use crate::services::device_service;
use crate::types::UserContext;

#[derive(Debug, Deserialize)]
pub(crate) struct IssueApiKeyRequest {
    pub name: String,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct IssueApiKeyResponse {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub secret: String,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub(crate) async fn issue_pat(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<IssueApiKeyRequest>,
) -> AdminResult<Response> {
    let issued =
        device_service::issue_pat(&pool, &user_ctx.user_id, &body.name, body.expires_at).await?;
    Ok(Json(IssueApiKeyResponse {
        id: issued.id,
        name: issued.name,
        key_prefix: issued.key_prefix,
        secret: issued.secret,
        created_at: issued.created_at,
        expires_at: issued.expires_at,
    })
    .into_response())
}

pub(crate) async fn revoke_pat(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminResult<Response> {
    device_service::revoke_pat(&pool, &user_ctx.user_id, &id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Debug, Deserialize)]
pub(crate) struct EnrollDeviceRequest {
    pub user_id: UserId,
    pub name: String,
    pub platform: String,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct EnrollDeviceResponse {
    pub id: String,
    pub user_id: UserId,
    pub name: String,
    pub key_prefix: String,
    pub secret: String,
    pub platform: String,
    pub hostname: String,
    pub created_at: Option<DateTime<Utc>>,
    pub enrolled_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub(crate) async fn enroll_device(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<EnrollDeviceRequest>,
) -> AdminResult<Response> {
    // Why: `admin`, not `is_console`. Enrolling a device mints a certificate
    // for an arbitrary account.
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    let target = body.user_id;
    let hostname = body.hostname.unwrap_or_default();
    let enrolled = device_service::enroll_device(
        &pool,
        &target,
        device_service::EnrollDeviceInput {
            name: &body.name,
            platform: &body.platform,
            hostname: &hostname,
            expires_at: body.expires_at,
        },
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(EnrollDeviceResponse {
            id: enrolled.id,
            user_id: enrolled.user_id,
            name: enrolled.name,
            key_prefix: enrolled.key_prefix,
            secret: enrolled.secret,
            platform: enrolled.platform,
            hostname: enrolled.hostname,
            created_at: enrolled.created_at,
            enrolled_at: enrolled.enrolled_at,
            expires_at: enrolled.expires_at,
        }),
    )
        .into_response())
}

pub(crate) async fn revoke_cert(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminResult<Response> {
    device_service::revoke_device_cert(&pool, &user_ctx.user_id, &id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// Why: the self-service revokers above match on `id AND user_id`, which is
// what makes a guessable token id safe to expose. The fleet page needs the
// opposite: an administrator disabling somebody else's credential, on a route
// the write tier already gates to the admin roles. It is a separate handler
// because that difference is the whole authorisation story, and burying it in
// a branch inside the self-service one would hide it.
pub(crate) async fn admin_revoke_credential(
    State(pool): State<Arc<PgPool>>,
    Path((kind, id)): Path<(String, String)>,
) -> AdminResult<Response> {
    let revoked = match kind.as_str() {
        "pats" => repositories::devices::pats::revoke_any_api_key(&pool, &id).await,
        "certs" => repositories::devices::certs::revoke_any_device_cert(&pool, &id).await,
        _ => {
            return Err(AdminError::NotFound(format!(
                "unknown credential kind: {kind}"
            )));
        },
    }?;

    if revoked {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Err(AdminError::NotFound(
            "no active credential with that id".to_owned(),
        ))
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SetCertExpiryRequest {
    #[serde(default)]
    pub valid_until: Option<DateTime<Utc>>,
}

// Why: sets or clears the window on someone else's certificate, on the same
// admin-gated tier as the fleet revoke. The window is not the revocation:
// the sweep stamps `revoked_at` once it passes, which is what core reads.
pub(crate) async fn admin_set_cert_expiry(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
    Json(body): Json<SetCertExpiryRequest>,
) -> AdminResult<Response> {
    if body.valid_until.is_some_and(|until| until <= Utc::now()) {
        return Err(AdminError::BadRequest(
            "valid_until must be in the future".to_owned(),
        ));
    }
    let set = repositories::devices::certs::set_device_cert_validity(&pool, &id, body.valid_until)
        .await?;
    if set {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Err(AdminError::NotFound(
            "no active certificate with that id".to_owned(),
        ))
    }
}
