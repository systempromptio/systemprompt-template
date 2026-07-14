use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::handlers::shared;
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
) -> Response {
    match device_service::issue_pat(&pool, &user_ctx.user_id, &body.name, body.expires_at).await {
        Ok(issued) => Json(IssueApiKeyResponse {
            id: issued.id,
            name: issued.name,
            key_prefix: issued.key_prefix,
            secret: issued.secret,
            created_at: issued.created_at,
            expires_at: issued.expires_at,
        })
        .into_response(),
        Err(e) => e.into_response(),
    }
}

pub(crate) async fn revoke_pat(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> Response {
    match device_service::revoke_pat(&pool, &user_ctx.user_id, &id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct EnrollDeviceRequest {
    pub user_id: String,
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
    pub user_id: String,
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
) -> Response {
    if !user_ctx.is_admin {
        return shared::error_response(StatusCode::FORBIDDEN, "Admin access required");
    }
    let target = UserId::new(body.user_id);
    let hostname = body.hostname.unwrap_or_default();
    match device_service::enroll_device(
        &pool,
        &target,
        device_service::EnrollDeviceInput {
            name: &body.name,
            platform: &body.platform,
            hostname: &hostname,
            expires_at: body.expires_at,
        },
    )
    .await
    {
        Ok(enrolled) => (
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
            .into_response(),
        Err(e) => e.into_response(),
    }
}

pub(crate) async fn revoke_cert(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> Response {
    match device_service::revoke_device_cert(&pool, &user_ctx.user_id, &id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}
