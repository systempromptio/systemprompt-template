//! HTTP handlers for secret storage, rotation, and short-lived resolution
//! tokens.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use systemprompt::identifiers::UserId;

use crate::error::AdminResult;
use crate::handlers::users::extract_user_from_cookie;
use crate::services::auth::validate_plugin_jwt;
use crate::services::secret_service;

use super::responses::{
    AuditLogEntry, AuditLogListResponse, ResolutionTokenResponse, ResultOkResponse,
    SecretsListResponse,
};

const RESOLUTION_TOKEN_EXPIRY_SECS: u32 = 300;

#[derive(serde::Deserialize, Debug)]
pub(crate) struct ResolveQuery {
    token: String,
}

pub(crate) async fn create_resolution_token_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    match create_resolution_token_inner(&pool, &plugin_id, &headers).await {
        Ok(resp) => resp,
        Err(e) => e.into_response(),
    }
}

async fn create_resolution_token_inner(
    pool: &PgPool,
    plugin_id: &str,
    headers: &HeaderMap,
) -> AdminResult<Response> {
    let subject = validate_plugin_jwt(headers)?;
    let user_id = UserId::new(&subject);
    let token = secret_service::create_resolution_token(pool, &user_id, plugin_id).await?;
    Ok(Json(ResolutionTokenResponse {
        token,
        expires_in: RESOLUTION_TOKEN_EXPIRY_SECS,
    })
    .into_response())
}

pub(crate) async fn resolve_secrets_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    Query(params): Query<ResolveQuery>,
) -> Response {
    match secret_service::resolve_secrets(&pool, &plugin_id, &params.token).await {
        Ok(secrets) => Json(SecretsListResponse { secrets }).into_response(),
        Err(e) => e.into_response(),
    }
}

pub(crate) async fn audit_log_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    match audit_log_inner(&pool, &plugin_id, &headers).await {
        Ok(resp) => resp,
        Err(e) => e.into_response(),
    }
}

async fn audit_log_inner(
    pool: &PgPool,
    plugin_id: &str,
    headers: &HeaderMap,
) -> AdminResult<Response> {
    let session = extract_user_from_cookie(headers)?;
    let entries = secret_service::list_audit_log(pool, &session.user_id, plugin_id).await?;
    let items: Vec<AuditLogEntry> = entries
        .into_iter()
        .map(|row| AuditLogEntry {
            id: row.id,
            var_name: row.var_name,
            action: row.action,
            actor_id: row.actor_id,
            ip_address: row.ip_address,
            created_at: row.created_at,
        })
        .collect();
    Ok(Json(AuditLogListResponse { entries: items }).into_response())
}

pub(crate) async fn rotate_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    match rotate_inner(&pool, &plugin_id, &headers).await {
        Ok(resp) => resp,
        Err(e) => e.into_response(),
    }
}

async fn rotate_inner(
    pool: &PgPool,
    plugin_id: &str,
    headers: &HeaderMap,
) -> AdminResult<Response> {
    let session = extract_user_from_cookie(headers)?;
    secret_service::rotate_user_keys(pool, &session.user_id, plugin_id).await?;
    Ok(Json(ResultOkResponse { result: "ok" }).into_response())
}
