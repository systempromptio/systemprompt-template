//! HTTP handlers for secret storage, rotation, and short-lived resolution
//! tokens.

use crate::env::AdminEnv;

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use systemprompt::identifiers::PluginId;

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
    Extension(env): Extension<AdminEnv>,
    Path(plugin_id): Path<PluginId>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let user_id = validate_plugin_jwt(&env, &headers).await?;
    let token = secret_service::create_resolution_token(&pool, &user_id, &plugin_id).await?;
    Ok(Json(ResolutionTokenResponse {
        token,
        expires_in: RESOLUTION_TOKEN_EXPIRY_SECS,
    })
    .into_response())
}

pub(crate) async fn resolve_secrets_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(env): Extension<AdminEnv>,
    Path(plugin_id): Path<PluginId>,
    Query(params): Query<ResolveQuery>,
) -> AdminResult<Response> {
    let secrets =
        secret_service::resolve_secrets(&pool, env.cipher(), &plugin_id, &params.token).await?;
    Ok(Json(SecretsListResponse { secrets }).into_response())
}

pub(crate) async fn audit_log_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(env): Extension<AdminEnv>,
    Path(plugin_id): Path<PluginId>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let session = extract_user_from_cookie(&env, &headers).await?;
    let entries = secret_service::list_audit_log(&pool, &session.user_id, &plugin_id).await?;
    let items: Vec<AuditLogEntry> = entries
        .into_iter()
        .map(|row| AuditLogEntry {
            id: row.id,
            var_name: row.var_name,
            action: row.action,
            actor_id: row.actor_id.as_str().to_owned(),
            ip_address: row.ip_address,
            created_at: row.created_at,
        })
        .collect();
    Ok(Json(AuditLogListResponse { entries: items }).into_response())
}

pub(crate) async fn rotate_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(env): Extension<AdminEnv>,
    Path(plugin_id): Path<PluginId>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let session = extract_user_from_cookie(&env, &headers).await?;
    secret_service::rotate_user_keys(&pool, env.cipher(), &session.user_id, &plugin_id).await?;
    Ok(Json(ResultOkResponse { result: "ok" }).into_response())
}
