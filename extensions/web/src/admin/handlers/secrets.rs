use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::models::auth::JwtAudience;
use systemprompt::models::{Config, SecretsBootstrap};
use systemprompt::oauth::validate_jwt_token;

use crate::admin::handlers::users::extract_user_from_cookie;
use crate::admin::repositories::secret_crypto;
use crate::admin::repositories::secret_keys;
use crate::admin::repositories::secret_resolve;

#[derive(serde::Deserialize)]
pub(crate) struct ResolveQuery {
    token: String,
}

pub(crate) async fn create_resolution_token_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let user_id = match validate_plugin_jwt(&headers) {
        Ok(id) => id,
        Err(r) => return r,
    };

    match secret_resolve::create_resolution_token(&pool, &user_id, &plugin_id).await {
        Ok(token) => Json(serde_json::json!({
            "token": token,
            "expires_in": 300
        }))
        .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to create resolution token");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create token"})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn resolve_secrets_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    Query(params): Query<ResolveQuery>,
) -> Response {
    let (user_id, token_plugin_id) =
        match secret_resolve::validate_and_consume_token(&pool, &params.token).await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "Token validation failed");
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"error": "Invalid or expired token"})),
                )
                    .into_response();
            }
        };

    if token_plugin_id != plugin_id {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Token plugin mismatch"})),
        )
            .into_response();
    }

    let master_key = match secret_crypto::load_master_key() {
        Ok(k) => k,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load master key");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal configuration error"})),
            )
                .into_response();
        }
    };

    match secret_resolve::resolve_secrets_for_plugin(&pool, &user_id, &plugin_id, &master_key).await
    {
        Ok(secrets) => Json(secrets).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to resolve secrets");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to resolve secrets"})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn audit_log_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let (user_id, _, _) = match extract_user_from_cookie(&headers) {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": e})),
            )
                .into_response()
        }
    };

    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, String)>(
        "SELECT id, var_name, action, actor_id, ip_address, \
         created_at::text FROM secret_audit_log \
         WHERE user_id = $1 AND plugin_id = $2 \
         ORDER BY created_at DESC LIMIT 100",
    )
    .bind(&user_id)
    .bind(&plugin_id)
    .fetch_all(pool.as_ref())
    .await;

    match rows {
        Ok(entries) => {
            let json: Vec<serde_json::Value> = entries
                .into_iter()
                .map(|(id, var_name, action, actor_id, ip, created_at)| {
                    serde_json::json!({
                        "id": id,
                        "var_name": var_name,
                        "action": action,
                        "actor_id": actor_id,
                        "ip_address": ip,
                        "created_at": created_at
                    })
                })
                .collect();
            Json(json).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to query audit log");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to load audit log"})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn rotate_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let (user_id, _, _) = match extract_user_from_cookie(&headers) {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": e})),
            )
                .into_response()
        }
    };

    let master_key = match secret_crypto::load_master_key() {
        Ok(k) => k,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load master key");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal configuration error"})),
            )
                .into_response();
        }
    };

    if let Err(e) = secret_keys::rotate_user_dek(&pool, &user_id, &master_key).await {
        tracing::error!(error = %e, user_id = %user_id, "DEK rotation failed");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Key rotation failed"})),
        )
            .into_response();
    }

    let audit_id = uuid::Uuid::new_v4().to_string();
    let _ = sqlx::query(
        "INSERT INTO secret_audit_log (id, user_id, plugin_id, var_name, action, actor_id) \
         VALUES ($1, $2, $3, '*', 'rotated', $2)",
    )
    .bind(&audit_id)
    .bind(&user_id)
    .bind(&plugin_id)
    .execute(pool.as_ref())
    .await;

    Json(serde_json::json!({"result": "ok"})).into_response()
}

#[allow(clippy::result_large_err)]
fn validate_plugin_jwt(headers: &HeaderMap) -> Result<String, Response> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing Authorization header"})),
            )
                .into_response()
        })?;

    let jwt_secret = SecretsBootstrap::jwt_secret().map_err(|e| {
        tracing::error!(error = %e, "Failed to load JWT secret");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Internal configuration error"})),
        )
            .into_response()
    })?;

    let jwt_issuer = Config::get()
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to load config");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal configuration error"})),
            )
                .into_response()
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
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid or expired token"})),
        )
            .into_response()
    })?;

    Ok(claims.sub.clone())
}
