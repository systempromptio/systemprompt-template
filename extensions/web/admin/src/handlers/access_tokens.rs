//! HTTP handlers for personal access token management.

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
use crate::services::access_token_service;
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
        access_token_service::issue_pat(&pool, &user_ctx.user_id, &body.name, body.expires_at)
            .await?;
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

// Why: unlike the self-service `issue_pat`, this issues for the body's target
// user rather than the session's own id — the admin check below is the only
// thing standing between any caller and another user's credential.
pub(crate) async fn issue_user_pat(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<String>,
    Json(body): Json<IssueApiKeyRequest>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()));
    }
    let target = UserId::new(user_id);
    let issued =
        access_token_service::issue_pat(&pool, &target, &body.name, body.expires_at).await?;
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

// Why: the console lists every account's tokens, so an admin revoking a
// stale one from that page must not be limited to their own. The owner is
// part of the path so the row's identity is asserted, not guessed.
pub(crate) async fn revoke_user_pat(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path((user_id, id)): Path<(String, String)>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()));
    }
    let target = UserId::new(user_id);
    access_token_service::revoke_pat(&pool, &target, &id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub(crate) async fn revoke_pat(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminResult<Response> {
    access_token_service::revoke_pat(&pool, &user_ctx.user_id, &id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
