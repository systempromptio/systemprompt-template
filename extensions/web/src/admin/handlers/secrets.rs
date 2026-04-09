use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::models::auth::JwtAudience;

const RESOLUTION_TOKEN_EXPIRY_SECS: u32 = 300;
use systemprompt::models::{Config, SecretsBootstrap};
use systemprompt::oauth::validate_jwt_token;

use systemprompt::identifiers::UserId;

use crate::admin::handlers::shared;
use crate::admin::handlers::users::extract_user_from_cookie;
use crate::admin::repositories::secret_audit;
use crate::admin::repositories::secret_crypto;
use crate::admin::repositories::secret_keys;
use crate::admin::repositories::secret_resolve;

use super::responses::{
    AuditLogEntry, AuditLogListResponse, ResolutionTokenResponse, ResultOkResponse,
    SecretsListResponse,
};

#[derive(serde::Deserialize, Debug)]
pub struct ResolveQuery {
    token: String,
}

pub async fn create_resolution_token_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let user_id = match validate_plugin_jwt(&headers) {
        Ok(id) => UserId::new(&id),
        Err(r) => return *r,
    };

    match secret_resolve::create_resolution_token(&pool, &user_id, &plugin_id).await {
        Ok(token) => Json(ResolutionTokenResponse {
            token,
            expires_in: RESOLUTION_TOKEN_EXPIRY_SECS,
        })
        .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to create resolution token");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token")
        }
    }
}

pub async fn resolve_secrets_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    Query(params): Query<ResolveQuery>,
) -> Response {
    match resolve_secrets_inner(&pool, &plugin_id, &params.token).await {
        Ok(resp) => resp,
        Err(resp) => resp,
    }
}

async fn resolve_secrets_inner(
    pool: &PgPool,
    plugin_id: &str,
    token: &str,
) -> Result<Response, Response> {
    let (user_id, token_plugin_id) =
        secret_resolve::validate_and_consume_token(pool, token)
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, "Token validation failed");
                shared::error_response(StatusCode::UNAUTHORIZED, "Invalid or expired token")
            })?;

    if token_plugin_id != plugin_id {
        return Err(shared::error_response(
            StatusCode::FORBIDDEN,
            "Token plugin mismatch",
        ));
    }

    let master_key = secret_crypto::load_master_key().map_err(|e| {
        tracing::error!(error = %e, "Failed to load master key");
        shared::error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal configuration error",
        )
    })?;

    let secrets = secret_resolve::resolve_secrets_for_plugin(
        pool,
        &UserId::new(&user_id),
        plugin_id,
        &master_key,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to resolve secrets");
        shared::error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to resolve secrets",
        )
    })?;

    Ok(Json(SecretsListResponse { secrets }).into_response())
}

pub async fn audit_log_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let session = match extract_user_from_cookie(&headers) {
        Ok(s) => s,
        Err(e) => return shared::error_response(StatusCode::UNAUTHORIZED, &e),
    };
    let user_id = session.user_id;

    match secret_audit::list_audit_log(&pool, &user_id, &plugin_id).await {
        Ok(entries) => {
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
            Json(AuditLogListResponse { entries: items }).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to query audit log");
            shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load audit log",
            )
        }
    }
}

pub async fn rotate_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    match rotate_inner(&pool, &plugin_id, &headers).await {
        Ok(resp) => resp,
        Err(resp) => resp,
    }
}

async fn rotate_inner(
    pool: &PgPool,
    plugin_id: &str,
    headers: &HeaderMap,
) -> Result<Response, Response> {
    let session = extract_user_from_cookie(headers)
        .map_err(|e| shared::error_response(StatusCode::UNAUTHORIZED, &e))?;
    let user_id = session.user_id;

    let master_key = secret_crypto::load_master_key().map_err(|e| {
        tracing::error!(error = %e, "Failed to load master key");
        shared::error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal configuration error",
        )
    })?;

    secret_keys::rotate_user_dek(pool, &user_id, &master_key)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, user_id = %user_id, "DEK rotation failed");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Key rotation failed")
        })?;

    if let Err(e) = secret_audit::insert_audit_entry(pool, &user_id, plugin_id, "rotated").await {
        tracing::warn!(error = %e, "Failed to insert secret audit log");
    }

    Ok(Json(ResultOkResponse { result: "ok" }).into_response())
}

fn validate_plugin_jwt(headers: &HeaderMap) -> Result<String, Box<Response>> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            shared::boxed_error_response(StatusCode::UNAUTHORIZED, "Missing Authorization header")
        })?;

    let jwt_secret = SecretsBootstrap::jwt_secret().map_err(|e| {
        tracing::error!(error = %e, "Failed to load JWT secret");
        shared::boxed_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal configuration error",
        )
    })?;

    let jwt_issuer = Config::get()
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to load config");
            shared::boxed_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal configuration error",
            )
        })?
        .jwt_issuer
        .clone();

    let claims = validate_jwt_token(
        token,
        jwt_secret,
        &jwt_issuer,
        &[JwtAudience::Resource("plugin".to_string())],
    )
    .map_err(|e| {
        tracing::warn!(error = %e, "Plugin JWT validation failed");
        shared::boxed_error_response(StatusCode::UNAUTHORIZED, "Invalid or expired token")
    })?;

    Ok(claims.sub.clone())
}
