use crate::repository::OAuthRepository;
use crate::services::webauthn::WebAuthnManager;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use systemprompt_core_logging::LogService;
use systemprompt_core_users::repository::UserRepository;
use webauthn_rs::prelude::*;

#[derive(Debug, Deserialize)]
pub struct StartAuthQuery {
    pub email: String,
    pub oauth_state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StartAuthResponse {
    #[serde(rename = "publicKey")]
    pub public_key: serde_json::Value,
    pub challenge_id: String,
}

#[derive(Debug, Serialize)]
pub struct AuthError {
    pub error: String,
    pub error_description: String,
}

#[allow(unused_qualifications)]
pub async fn start_auth(
    Query(params): Query<StartAuthQuery>,
    State(ctx): State<systemprompt_core_system::AppContext>,
) -> impl IntoResponse {
    let oauth_repo = OAuthRepository::new(ctx.db_pool().clone());
    let user_repo = UserRepository::new(ctx.db_pool().clone());
    let log_service = LogService::system(ctx.db_pool().clone());

    let webauthn_service =
        match WebAuthnManager::get_or_create_service(oauth_repo, user_repo, log_service).await {
            Ok(service) => service,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AuthError {
                        error: "server_error".to_string(),
                        error_description: format!("Failed to initialize WebAuthn: {e}"),
                    }),
                )
                    .into_response();
            },
        };

    match webauthn_service
        .start_authentication(&params.email, params.oauth_state)
        .await
    {
        Ok((challenge, challenge_id)) => {
            // Extract publicKey from RequestChallengeResponse to match W3C WebAuthn standard
            let challenge_json = serde_json::to_value(&challenge)
                .map_err(|e| anyhow::anyhow!("Failed to serialize challenge: {e}"))
                .unwrap();

            let public_key = challenge_json
                .get("publicKey")
                .ok_or_else(|| anyhow::anyhow!("Missing publicKey in challenge"))
                .unwrap()
                .clone();

            (
                StatusCode::OK,
                Json(StartAuthResponse {
                    public_key,
                    challenge_id,
                }),
            )
                .into_response()
        },
        Err(e) => {
            let status_code = if e.to_string().contains("User not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };

            (
                status_code,
                Json(AuthError {
                    error: "authentication_failed".to_string(),
                    error_description: e.to_string(),
                }),
            )
                .into_response()
        },
    }
}

#[derive(Debug, Deserialize)]
pub struct FinishAuthRequest {
    pub challenge_id: String,
    pub credential: PublicKeyCredential,
}

#[derive(Debug, Serialize)]
pub struct FinishAuthResponse {
    pub user_id: String,
    pub oauth_state: Option<String>,
    pub success: bool,
}

pub async fn finish_auth(
    State(ctx): State<systemprompt_core_system::AppContext>,
    Json(request): Json<FinishAuthRequest>,
) -> impl IntoResponse {
    let oauth_repo = OAuthRepository::new(ctx.db_pool().clone());
    let user_repo = UserRepository::new(ctx.db_pool().clone());
    let log_service = LogService::system(ctx.db_pool().clone());

    let webauthn_service =
        match WebAuthnManager::get_or_create_service(oauth_repo, user_repo, log_service).await {
            Ok(service) => service,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AuthError {
                        error: "server_error".to_string(),
                        error_description: format!("Failed to initialize WebAuthn: {e}"),
                    }),
                )
                    .into_response();
            },
        };

    match webauthn_service
        .finish_authentication(&request.challenge_id, &request.credential)
        .await
    {
        Ok((user_id, oauth_state)) => (
            StatusCode::OK,
            Json(FinishAuthResponse {
                user_id,
                oauth_state,
                success: true,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::UNAUTHORIZED,
            Json(AuthError {
                error: "authentication_failed".to_string(),
                error_description: e.to_string(),
            }),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct DevAuthQuery {
    pub email: String,
    pub oauth_state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DevAuthResponse {
    pub user_id: String,
    pub oauth_state: Option<String>,
    pub success: bool,
}

fn is_dev_mode() -> bool {
    std::env::var("DANGEROUSLY_BYPASS_OAUTH")
        .map(|s| s.to_lowercase() == "true")
        .unwrap_or(false)
}

#[allow(unused_qualifications)]
pub async fn dev_auth(
    Query(params): Query<DevAuthQuery>,
    State(ctx): State<systemprompt_core_system::AppContext>,
) -> impl IntoResponse {
    if !is_dev_mode() {
        return (
            StatusCode::FORBIDDEN,
            Json(AuthError {
                error: "forbidden".to_string(),
                error_description: "Development authentication not available in production"
                    .to_string(),
            }),
        )
            .into_response();
    }

    let user_repo = UserRepository::new(ctx.db_pool().clone());
    let log_service = LogService::system(ctx.db_pool().clone());

    match user_repo.find_by_email(&params.email).await {
        Ok(Some(user)) => {
            let _ = log_service
                .info(
                    "webauthn",
                    &format!("DEV: Email-based authentication for: {}", params.email),
                )
                .await;

            (
                StatusCode::OK,
                Json(DevAuthResponse {
                    user_id: user.uuid,
                    oauth_state: params.oauth_state,
                    success: true,
                }),
            )
                .into_response()
        },
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(AuthError {
                error: "user_not_found".to_string(),
                error_description: format!("User not found: {}", params.email),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthError {
                error: "server_error".to_string(),
                error_description: format!("Database error: {e}"),
            }),
        )
            .into_response(),
    }
}
