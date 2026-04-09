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

use systemprompt::identifiers::{Email, UserId};

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::handlers::shared;
use crate::repositories;
use crate::types::CreateUserRequest;
use crate::types::EventsQuery;
use crate::types::UpdateUserRequest;
use crate::types::UserContext;

use super::responses::{EventsListResponse, UsersListResponse};

pub fn extract_user_from_cookie(
    headers: &HeaderMap,
) -> Result<crate::types::CookieSession, String> {
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

    let email =
        Email::try_new(claims.email.clone()).map_err(|e| format!("Invalid email in JWT: {e}"))?;

    Ok(crate::types::CookieSession {
        user_id: UserId::new(claims.sub),
        username: claims.username,
        email,
    })
}

pub async fn dashboard_handler(State(pool): State<Arc<PgPool>>) -> Response {
    match repositories::get_dashboard_data(&pool, "7 days", "4 hours", "today", "7d").await {
        Ok(data) => Json(data).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to load dashboard data");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn list_users_handler(State(pool): State<Arc<PgPool>>) -> Response {
    match repositories::list_users(&pool).await {
        Ok(users) => Json(UsersListResponse { users }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list users");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn user_detail_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id_raw): Path<String>,
) -> Response {
    let user_id = UserId::new(user_id_raw);
    match repositories::find_user_detail(&pool, &user_id).await {
        Ok(Some(detail)) => Json(detail).into_response(),
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "User not found"),
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Failed to get user detail");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn user_usage_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id_raw): Path<String>,
) -> Response {
    let user_id = UserId::new(user_id_raw);
    match repositories::get_user_usage(&pool, &user_id).await {
        Ok(events) => Json(EventsListResponse { events }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Failed to get user usage");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn create_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<CreateUserRequest>,
) -> Response {
    if !user_ctx.is_admin {
        return shared::error_response(StatusCode::FORBIDDEN, "Admin access required");
    }
    match repositories::create_user(&pool, &body).await {
        Ok(user) => {
            let p = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let new_user_id = user.user_id.clone();
            let name = user
                .display_name
                .clone()
                .unwrap_or_else(|| user.user_id.as_str().to_string());
            tokio::spawn(async move {
                activity::record(
                    &p,
                    NewActivity::entity_created(
                        &uid,
                        ActivityEntity::User,
                        new_user_id.as_str(),
                        &name,
                    ),
                )
                .await;
            });
            (StatusCode::CREATED, Json(user)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create user");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn update_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id_raw): Path<String>,
    Json(body): Json<UpdateUserRequest>,
) -> Response {
    let user_id = UserId::new(user_id_raw);
    if !user_ctx.is_admin {
        return shared::error_response(StatusCode::FORBIDDEN, "Admin access required");
    }
    match repositories::update_user(&pool, &user_id, &body).await {
        Ok(Some(user)) => {
            let p = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let target_user_id = user.user_id.clone();
            let name = user
                .display_name
                .clone()
                .unwrap_or_else(|| user.user_id.as_str().to_string());
            tokio::spawn(async move {
                activity::record(
                    &p,
                    NewActivity::entity_updated(
                        &uid,
                        ActivityEntity::User,
                        target_user_id.as_str(),
                        &name,
                    ),
                )
                .await;
            });
            Json(user).into_response()
        }
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "User not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update user");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn delete_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id_raw): Path<String>,
) -> Response {
    let user_id = UserId::new(user_id_raw);
    if !user_ctx.is_admin {
        return shared::error_response(StatusCode::FORBIDDEN, "Admin access required");
    }
    match repositories::delete_user(&pool, &user_id).await {
        Ok(true) => {
            let p = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let target = user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &p,
                    NewActivity::entity_deleted(
                        &uid,
                        ActivityEntity::User,
                        target.as_str(),
                        target.as_str(),
                    ),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "User not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete user");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn list_events_handler(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<EventsQuery>,
) -> Response {
    match repositories::list_events(&pool, &query).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list events");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}
