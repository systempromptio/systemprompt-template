use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::models::auth::JwtAudience;
use systemprompt::models::{Config, SecretsBootstrap};
use systemprompt::oauth::validate_jwt_token;

use crate::admin::repositories;
use crate::admin::types::CreateUserRequest;
use crate::admin::types::EventsQuery;
use crate::admin::types::UpdateUserRequest;
use crate::admin::types::UserContext;

pub(crate) fn extract_user_from_cookie(
    headers: &HeaderMap,
) -> Result<(String, String, String), String> {
    let cookie_header = headers
        .get("cookie")
        .ok_or("No cookie header")?
        .to_str()
        .map_err(|e| format!("Invalid cookie header: {e}"))?;

    let token = cookie_header
        .split(';')
        .find_map(|c| c.trim().strip_prefix("access_token="))
        .ok_or("No access_token cookie")?;

    if token.is_empty() {
        return Err("Empty access_token cookie".to_string());
    }

    let jwt_secret =
        SecretsBootstrap::jwt_secret().map_err(|e| format!("Failed to load JWT secret: {e}"))?;
    let jwt_issuer = Config::get()
        .map_err(|e| format!("Failed to load config: {e}"))?
        .jwt_issuer
        .clone();

    let claims = validate_jwt_token(token, jwt_secret, &jwt_issuer, &[JwtAudience::Api])
        .map_err(|e| format!("JWT validation failed: {e}"))?;

    Ok((
        claims.sub.clone(),
        claims.username.clone(),
        claims.email.clone(),
    ))
}

pub(crate) async fn dashboard_handler(State(pool): State<Arc<PgPool>>) -> Response {
    match repositories::get_dashboard_data(&pool, None, "7 days").await {
        Ok(data) => Json(data).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to load dashboard data");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn list_users_handler(State(pool): State<Arc<PgPool>>) -> Response {
    match repositories::list_users(&pool, None).await {
        Ok(users) => Json(users).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list users");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn user_detail_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<String>,
) -> Response {
    match repositories::get_user_detail(&pool, &user_id).await {
        Ok(Some(detail)) => Json(detail).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "User not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Failed to get user detail");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn user_usage_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<String>,
) -> Response {
    match repositories::get_user_usage(&pool, &user_id).await {
        Ok(events) => Json(events).into_response(),
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Failed to get user usage");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn create_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<CreateUserRequest>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        )
            .into_response();
    }
    match repositories::create_user(&pool, &body).await {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to create user");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn update_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id): Path<String>,
    Json(body): Json<UpdateUserRequest>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        )
            .into_response();
    }
    match repositories::update_user(&pool, &user_id, &body).await {
        Ok(Some(user)) => Json(user).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "User not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update user");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn delete_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        )
            .into_response();
    }
    match repositories::delete_user(&pool, &user_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "User not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete user");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn list_events_handler(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<EventsQuery>,
) -> Response {
    match repositories::list_events(&pool, &query, None).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list events");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
