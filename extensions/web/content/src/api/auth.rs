//! Session cookie issue and clear for the public content surface.

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;
use serde::Deserialize;

use crate::api::OkResponse;

#[derive(Deserialize, Debug)]
pub struct SetSessionRequest {
    pub access_token: String,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
}

/// Whether the session cookies carry `Secure`: the profile's `use_https`.
#[derive(Clone, Copy, Debug)]
pub struct CookieSecure(pub bool);

const fn secure_flag(secure: CookieSecure) -> &'static str {
    if secure.0 { "; Secure" } else { "" }
}

fn build_session_cookies(body: &SetSessionRequest, secure: CookieSecure) -> HeaderMap {
    let max_age = body.expires_in.unwrap_or(3600);
    let secure_flag = secure_flag(secure);
    let access_cookie = format!(
        "access_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}",
        body.access_token, max_age, secure_flag
    );

    let mut headers = HeaderMap::new();
    if let Ok(val) = access_cookie.parse() {
        headers.insert(SET_COOKIE, val);
    }

    if let Some(ref refresh) = body.refresh_token {
        let refresh_max_age = 30 * 24 * 3600;
        let refresh_cookie = format!(
            "refresh_token={refresh}; Path=/api/public/auth; HttpOnly; SameSite=Lax; Max-Age={refresh_max_age}{secure_flag}",
        );
        if let Ok(val) = refresh_cookie.parse() {
            headers.append(SET_COOKIE, val);
        }
    }

    headers
}

fn build_clear_cookies(secure: CookieSecure) -> HeaderMap {
    let secure_flag = secure_flag(secure);
    let access_cookie =
        format!("access_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{secure_flag}");
    let refresh_cookie = format!(
        "refresh_token=; Path=/api/public/auth; HttpOnly; SameSite=Lax; Max-Age=0{secure_flag}"
    );

    let mut headers = HeaderMap::new();
    if let Ok(val) = access_cookie.parse() {
        headers.insert(SET_COOKIE, val);
    }
    if let Ok(val) = refresh_cookie.parse() {
        headers.append(SET_COOKIE, val);
    }

    headers
}

pub async fn set_session(
    State(secure): State<CookieSecure>,
    Json(body): Json<SetSessionRequest>,
) -> (HeaderMap, Json<OkResponse>) {
    std::future::ready(()).await;
    let headers = build_session_cookies(&body, secure);
    (headers, Json(OkResponse { ok: true }))
}

pub async fn clear_session(State(secure): State<CookieSecure>) -> (HeaderMap, Json<OkResponse>) {
    std::future::ready(()).await;
    let headers = build_clear_cookies(secure);
    (headers, Json(OkResponse { ok: true }))
}
