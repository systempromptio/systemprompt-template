//! HTTP handlers for user administration.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use systemprompt::models::Config;
use systemprompt::models::auth::JwtAudience;
use systemprompt::oauth::validate_jwt_token;

use systemprompt::identifiers::{Email, UserId};

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::error::{AdminError, AdminResult};
use crate::repositories;
use crate::types::{CreateUserRequest, EventsQuery, UpdateUserRequest, UserContext};

use super::responses::{EventsListResponse, UsersListResponse};

pub(crate) fn extract_user_from_cookie(
    headers: &HeaderMap,
) -> Result<crate::types::CookieSession, AdminError> {
    let token = extract_token_from_headers(headers)?;

    let jwt_issuer = Config::get()?.jwt_issuer.clone();

    let claims = validate_jwt_token(&token, &jwt_issuer, &[JwtAudience::Api])?;

    let email = Email::try_new(claims.email.clone()).map_err(AdminError::unauthenticated)?;

    Ok(crate::types::CookieSession {
        user_id: UserId::new(claims.sub),
        username: claims.username,
        email,
    })
}

fn extract_token_from_headers(headers: &HeaderMap) -> Result<String, AdminError> {
    if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok())
        && let Some(token) = auth
            .strip_prefix("Bearer ")
            .or_else(|| auth.strip_prefix("bearer "))
    {
        let t = token.trim();
        if !t.is_empty() {
            return Ok(t.to_owned());
        }
    }

    let cookie_header = headers
        .get("cookie")
        .ok_or_else(|| AdminError::Unauthorized("No cookie or Authorization header".to_owned()))?
        .to_str()
        .map_err(|e| AdminError::Unauthorized(format!("Invalid cookie header: {e}")))?;

    let token = cookie_header
        .split(';')
        .find_map(|c| c.trim().strip_prefix("access_token="))
        .ok_or_else(|| {
            AdminError::Unauthorized("No access_token cookie or Authorization: Bearer".to_owned())
        })?;

    if token.is_empty() {
        return Err(AdminError::Unauthorized(
            "Empty access_token cookie".to_owned(),
        ));
    }
    Ok(token.to_owned())
}

pub(crate) async fn dashboard_handler(State(pool): State<Arc<PgPool>>) -> AdminResult<Response> {
    let data =
        repositories::dashboard::get_dashboard_data(&pool, "7 days", "4 hours", "today", "7d")
            .await?;
    Ok(Json(data).into_response())
}

pub(crate) async fn list_users_handler(State(pool): State<Arc<PgPool>>) -> AdminResult<Response> {
    let users = repositories::users::queries::list_users(&pool).await?;
    Ok(Json(UsersListResponse { users }).into_response())
}

pub(crate) async fn user_detail_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id_raw): Path<String>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id_raw);
    let detail = repositories::users::queries::find_user_detail(&pool, &user_id)
        .await?
        .ok_or_else(|| AdminError::NotFound("User not found".to_owned()))?;
    Ok(Json(detail).into_response())
}

pub(crate) async fn user_usage_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id_raw): Path<String>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id_raw);
    let events = repositories::users::queries::list_user_usage(&pool, &user_id).await?;
    Ok(Json(EventsListResponse { events }).into_response())
}

pub(crate) async fn create_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<CreateUserRequest>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    let user = repositories::users::mutations::create_user(&pool, &body).await?;
    let p = Arc::clone(&pool);
    let uid = user_ctx.user_id.clone();
    let new_user_id = user.user_id.clone();
    let name = user
        .display_name
        .clone()
        .unwrap_or_else(|| user.user_id.as_str().to_owned());
    tokio::spawn(async move {
        activity::record(
            &p,
            NewActivity::entity_created(&uid, ActivityEntity::User, new_user_id.as_str(), &name),
        )
        .await;
    });
    Ok((StatusCode::CREATED, Json(user)).into_response())
}

pub(crate) async fn update_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id_raw): Path<String>,
    Json(body): Json<UpdateUserRequest>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id_raw);
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    let user = repositories::users::mutations::update_user(&pool, &user_id, &body)
        .await?
        .ok_or_else(|| AdminError::NotFound("User not found".to_owned()))?;
    let p = Arc::clone(&pool);
    let uid = user_ctx.user_id.clone();
    let target_user_id = user.user_id.clone();
    let name = user
        .display_name
        .clone()
        .unwrap_or_else(|| user.user_id.as_str().to_owned());
    tokio::spawn(async move {
        activity::record(
            &p,
            NewActivity::entity_updated(&uid, ActivityEntity::User, target_user_id.as_str(), &name),
        )
        .await;
    });
    Ok(Json(user).into_response())
}

pub(crate) async fn delete_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id_raw): Path<String>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id_raw);
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    if !repositories::users::mutations::delete_user(&pool, &user_id).await? {
        return Err(AdminError::NotFound("User not found".to_owned()));
    }
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
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub(crate) async fn list_events_handler(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<EventsQuery>,
) -> AdminResult<Response> {
    let response = repositories::dashboard::list_events(&pool, &query).await?;
    Ok(Json(response).into_response())
}
