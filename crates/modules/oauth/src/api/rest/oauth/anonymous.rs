use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::services::cimd::ClientValidator;
use crate::services::SessionCreationService;
use systemprompt_core_system::AppContext;
use systemprompt_core_users::UserRepository;
use systemprompt_identifiers::ClientId;
use systemprompt_models::auth::TokenType;

#[derive(Debug, Serialize)]
pub struct AnonymousTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub session_id: String,
    pub user_id: String,
    pub client_id: String,
    pub client_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AnonymousTokenRequest {
    #[serde(default = "default_client_id")]
    pub client_id: String,
    #[serde(default)]
    pub redirect_uri: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

fn default_client_id() -> String {
    "sp_web".to_string()
}

pub async fn generate_anonymous_token(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    Json(req): Json<AnonymousTokenRequest>,
) -> impl IntoResponse {
    let expires_in = 24 * 3600;
    let client_id = ClientId::new(req.client_id);
    let validator = ClientValidator::new(ctx.db_pool().clone());

    let validation = match validator
        .validate_client(&client_id, req.redirect_uri.as_deref())
        .await
    {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_client",
                    "error_description": format!("Client validation failed: {e}")
                })),
            )
                .into_response();
        },
    };

    let client_type = validation.client_type();

    let session_service = SessionCreationService::new(
        ctx.analytics_service().clone(),
        UserRepository::new(ctx.db_pool().clone()),
    );

    match session_service
        .create_anonymous_session(&headers, None, &client_id, ctx.jwt_secret())
        .await
    {
        Ok(session_info) => {
            let cookie = format!(
                "access_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
                session_info.jwt_token, expires_in
            );

            let mut response = (
                StatusCode::OK,
                Json(AnonymousTokenResponse {
                    access_token: session_info.jwt_token,
                    token_type: TokenType::Bearer.to_string(),
                    expires_in,
                    session_id: session_info.session_id.as_str().to_string(),
                    user_id: session_info.user_id.as_str().to_string(),
                    client_id: client_id.as_str().to_string(),
                    client_type: client_type.as_str().to_string(),
                }),
            )
                .into_response();

            response
                .headers_mut()
                .insert(header::SET_COOKIE, cookie.parse().unwrap());

            response
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "server_error",
                "error_description": format!("Failed to create session: {e}")
            })),
        )
            .into_response(),
    }
}
