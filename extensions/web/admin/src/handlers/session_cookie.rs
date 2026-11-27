//! Session cookie issue and clear behind `/api/public/auth/session`, served on
//! every flavour: the console login sets the cookie through it.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, State};
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{IntoResponse, Response};
use cookie::time::Duration;
use cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::models::auth::JwtAudience;

use crate::env::AdminEnv;
use crate::error::{AdminError, AdminResult};
use crate::repositories::users::queries::find_user_access_profile;
use crate::types::UserContext;

#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct OkResponse {
    pub(crate) ok: bool,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct SetSessionRequest {
    pub(crate) access_token: String,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CookieSecure(pub(crate) bool);

#[derive(Clone)]
pub(crate) struct SessionState {
    pub(crate) pool: Arc<PgPool>,
    pub(crate) secure: CookieSecure,
}

fn access_cookie(value: &str, max_age: i64, secure: CookieSecure) -> AdminResult<HeaderValue> {
    let cookie = Cookie::build(("access_token", value))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(max_age))
        .secure(secure.0)
        .build();
    HeaderValue::from_str(&cookie.to_string()).map_err(AdminError::unauthenticated)
}

fn cookie_headers(cookie: HeaderValue) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie);
    headers
}

pub(crate) async fn set_session(
    State(state): State<SessionState>,
    Extension(env): Extension<AdminEnv>,
    Json(body): Json<SetSessionRequest>,
) -> AdminResult<Response> {
    let (session, exp) =
        super::users::session_from_token(&env, &body.access_token, &[JwtAudience::Api]).await?;
    let account = find_user_access_profile(&state.pool, &session.user_id)
        .await
        .map_err(AdminError::internal)?;
    UserContext::signed_in(session, account).map_err(AdminError::unauthenticated)?;
    let max_age = (exp - chrono::Utc::now().timestamp()).max(0);
    let headers = cookie_headers(access_cookie(&body.access_token, max_age, state.secure)?);
    Ok((headers, Json(OkResponse { ok: true })).into_response())
}

pub(crate) async fn clear_session(State(state): State<SessionState>) -> AdminResult<Response> {
    std::future::ready(()).await;
    let headers = cookie_headers(access_cookie("", 0, state.secure)?);
    Ok((headers, Json(OkResponse { ok: true })).into_response())
}
