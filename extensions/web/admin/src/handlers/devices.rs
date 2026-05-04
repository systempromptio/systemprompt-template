use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::services::device_service;
use crate::types::UserContext;

#[derive(Debug, Deserialize)]
pub struct IssueApiKeyRequest {
    pub name: String,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct IssueApiKeyResponse {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub secret: String,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub async fn issue_pat(
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

pub async fn revoke_pat(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> Response {
    match device_service::revoke_pat(&pool, &user_ctx.user_id, &id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn revoke_cert(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> Response {
    match device_service::revoke_device_cert(&pool, &user_ctx.user_id, &id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}
