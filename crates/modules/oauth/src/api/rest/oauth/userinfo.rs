use crate::services::validate_jwt_token;
use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]

pub struct UserinfoResponse {
    pub sub: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub user_type: Option<String>,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]

pub struct UserinfoError {
    pub error: String,
    pub error_description: Option<String>,
}

pub async fn handle_userinfo(
    State(_ctx): State<systemprompt_core_system::AppContext>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let Some(token) = extract_bearer_token(&headers) else {
        let error = UserinfoError {
            error: "invalid_request".to_string(),
            error_description: Some("Missing or invalid Authorization header".to_string()),
        };
        return (StatusCode::UNAUTHORIZED, Json(error)).into_response();
    };

    if let Ok(userinfo) = get_userinfo(&token).await { (StatusCode::OK, Json(userinfo)).into_response() } else {
        let error = UserinfoError {
            error: "invalid_token".to_string(),
            error_description: Some(
                "The access token provided is expired, revoked, malformed, or invalid"
                    .to_string(),
            ),
        };
        (StatusCode::UNAUTHORIZED, Json(error)).into_response()
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get("authorization")?;
    let auth_str = auth_header.to_str().ok()?;

    auth_str.strip_prefix("Bearer ").map(ToString::to_string)
}

async fn get_userinfo(token: &str) -> Result<UserinfoResponse> {
    let jwt_secret = &systemprompt_core_system::Config::global().jwt_secret;
    let claims = validate_jwt_token(token, jwt_secret)?;

    Ok(UserinfoResponse {
        sub: claims.sub.clone(),
        username: Some(claims.username.clone()),
        email: Some(claims.email.clone()),
        user_type: Some(claims.user_type.to_string()),
        roles: Some(claims.get_scopes()),
    })
}
