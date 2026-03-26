use axum::http::header::SET_COOKIE;
use axum::http::HeaderMap;
use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SetSessionRequest {
    pub access_token: String,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
}

fn is_secure_request(headers: &HeaderMap) -> bool {
    if let Some(proto) = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
    {
        return proto == "https";
    }
    if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) {
        return origin.starts_with("https://");
    }
    false
}

pub async fn set_session(
    req_headers: HeaderMap,
    Json(body): Json<SetSessionRequest>,
) -> (HeaderMap, Json<serde_json::Value>) {
    let max_age = body.expires_in.unwrap_or(3600);
    let secure_flag = if is_secure_request(&req_headers) {
        "; Secure"
    } else {
        ""
    };
    let access_cookie = format!(
        "access_token={}; Path=/; SameSite=Lax; Max-Age={}{}",
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

    (headers, Json(serde_json::json!({ "ok": true })))
}

pub async fn clear_session() -> (HeaderMap, Json<serde_json::Value>) {
    let access_cookie = "access_token=; Path=/; SameSite=Lax; Max-Age=0";
    let refresh_cookie = "refresh_token=; Path=/api/public/auth; HttpOnly; SameSite=Lax; Max-Age=0";

    let mut headers = HeaderMap::new();
    if let Ok(val) = access_cookie.parse() {
        headers.insert(SET_COOKIE, val);
    }
    if let Ok(val) = refresh_cookie.parse() {
        headers.append(SET_COOKIE, val);
    }

    (headers, Json(serde_json::json!({ "ok": true })))
}
