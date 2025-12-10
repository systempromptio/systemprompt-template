use crate::repository::OAuthRepository;
use crate::services::webauthn::WebAuthnManager;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use systemprompt_core_logging::LogService;
use systemprompt_core_users::repository::UserRepository;
use webauthn_rs::prelude::*;

#[derive(Debug, Deserialize)]
pub struct StartRegisterQuery {
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
}

impl StartRegisterQuery {
    fn validate(&self) -> Result<(), String> {
        if self.username.trim().is_empty() {
            return Err("Username is required and cannot be empty".to_string());
        }

        if self.username.len() > 50 {
            return Err("Username must be less than 50 characters".to_string());
        }

        if !self
            .username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(
                "Username can only contain letters, numbers, underscores, and hyphens".to_string(),
            );
        }

        if self.email.trim().is_empty() {
            return Err("Email is required and cannot be empty".to_string());
        }

        if !self.email.contains('@') || !self.email.contains('.') {
            return Err("Email must be a valid email address".to_string());
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct RegisterError {
    pub error: String,
    pub error_description: String,
}

#[allow(unused_qualifications)]
pub async fn start_register(
    Query(params): Query<StartRegisterQuery>,
    State(ctx): State<systemprompt_core_system::AppContext>,
) -> impl IntoResponse {
    if let Err(validation_error) = params.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(RegisterError {
                error: "invalid_request".to_string(),
                error_description: validation_error,
            }),
        )
            .into_response();
    }

    let oauth_repo = OAuthRepository::new(ctx.db_pool().clone());
    let user_repo = UserRepository::new(ctx.db_pool().clone());
    let log_service = LogService::system(ctx.db_pool().clone());

    let webauthn_service =
        match WebAuthnManager::get_or_create_service(oauth_repo, user_repo, log_service).await {
            Ok(service) => service,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(RegisterError {
                        error: "server_error".to_string(),
                        error_description: format!("Failed to initialize WebAuthn: {e}"),
                    }),
                )
                    .into_response();
            },
        };

    match webauthn_service
        .start_registration(&params.username, &params.email, params.full_name.as_deref())
        .await
    {
        Ok((challenge, challenge_id)) => {
            let mut challenge_json = serde_json::to_value(&challenge)
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(RegisterError {
                            error: "server_error".to_string(),
                            error_description: format!("Failed to serialize challenge: {e}"),
                        }),
                    )
                        .into_response()
                })
                .unwrap();

            if let Some(public_key) = challenge_json.get_mut("publicKey") {
                if let Some(authenticator_selection) = public_key.get_mut("authenticatorSelection")
                {
                    if let Some(obj) = authenticator_selection.as_object_mut() {
                        obj.remove("authenticatorAttachment");
                    }
                }
            }

            let mut headers = HeaderMap::new();
            let header_value = HeaderValue::from_str(&challenge_id).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(RegisterError {
                        error: "server_error".to_string(),
                        error_description: format!("Invalid challenge ID format: {e}"),
                    }),
                )
                    .into_response()
            });

            match header_value {
                Ok(val) => {
                    headers.insert(HeaderName::from_static("x-challenge-id"), val);
                    (StatusCode::OK, headers, Json(challenge_json)).into_response()
                },
                Err(response) => response,
            }
        },
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(RegisterError {
                error: "registration_failed".to_string(),
                error_description: e.to_string(),
            }),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct FinishRegisterRequest {
    pub challenge_id: String,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub credential: RegisterPublicKeyCredential,
    #[serde(default)]
    pub session_id: Option<String>,
}

impl FinishRegisterRequest {
    fn validate(&self) -> Result<(), String> {
        if self.challenge_id.trim().is_empty() {
            return Err("Challenge ID is required".to_string());
        }

        if self.username.trim().is_empty() {
            return Err("Username is required and cannot be empty".to_string());
        }

        if self.username.len() > 50 {
            return Err("Username must be less than 50 characters".to_string());
        }

        if !self
            .username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(
                "Username can only contain letters, numbers, underscores, and hyphens".to_string(),
            );
        }

        if self.email.trim().is_empty() {
            return Err("Email is required and cannot be empty".to_string());
        }

        if !self.email.contains('@') || !self.email.contains('.') {
            return Err("Email must be a valid email address".to_string());
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct FinishRegisterResponse {
    pub user_id: String,
    pub success: bool,
}

pub async fn finish_register(
    State(ctx): State<systemprompt_core_system::AppContext>,
    Json(request): Json<FinishRegisterRequest>,
) -> impl IntoResponse {
    if let Err(validation_error) = request.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(RegisterError {
                error: "invalid_request".to_string(),
                error_description: validation_error,
            }),
        )
            .into_response();
    }

    let oauth_repo = OAuthRepository::new(ctx.db_pool().clone());
    let user_repo = UserRepository::new(ctx.db_pool().clone());
    let log_service = LogService::system(ctx.db_pool().clone());

    let webauthn_service =
        match WebAuthnManager::get_or_create_service(oauth_repo, user_repo, log_service).await {
            Ok(service) => service,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(RegisterError {
                        error: "server_error".to_string(),
                        error_description: format!("Failed to initialize WebAuthn: {e}"),
                    }),
                )
                    .into_response();
            },
        };

    match webauthn_service
        .finish_registration(
            &request.challenge_id,
            &request.username,
            &request.email,
            request.full_name.as_deref(),
            &request.credential,
        )
        .await
    {
        Ok(user_id) => {
            if let Some(session_id) = &request.session_id {
                use systemprompt_core_system::repository::AnalyticsSessionRepository;

                let session_repo = AnalyticsSessionRepository::new(ctx.db_pool().clone());
                let session_logger = LogService::system(ctx.db_pool().clone());

                match session_repo.get_session(session_id).await {
                    Ok(Some(session)) => {
                        let old_user_id = session.user_id.unwrap_or_default();

                        match session_repo
                            .migrate_session_to_registered_user(session_id, &old_user_id, &user_id)
                            .await
                        {
                            Ok(result) => {
                                session_logger
                                    .info(
                                        "webauthn_register",
                                        &format!(
                                            "Successfully migrated user data: session={}, \
                                             old_user={}, new_user={}, records={}",
                                            session_id,
                                            old_user_id,
                                            user_id,
                                            result.total_records_migrated()
                                        ),
                                    )
                                    .await
                                    .ok();
                            },
                            Err(e) => {
                                session_logger
                                    .error(
                                        "webauthn_register",
                                        &format!(
                                            "Failed to migrate session {session_id} from \
                                             {old_user_id} to {user_id}: {e}"
                                        ),
                                    )
                                    .await
                                    .ok();
                            },
                        }
                    },
                    Ok(None) => {
                        session_logger
                            .warn(
                                "webauthn_register",
                                &format!("Session {session_id} not found for migration"),
                            )
                            .await
                            .ok();
                    },
                    Err(e) => {
                        session_logger
                            .error(
                                "webauthn_register",
                                &format!(
                                    "Failed to retrieve session {session_id} for migration: {e}"
                                ),
                            )
                            .await
                            .ok();
                    },
                }
            }

            (
                StatusCode::OK,
                Json(FinishRegisterResponse {
                    user_id,
                    success: true,
                }),
            )
                .into_response()
        },
        Err(e) => {
            let error_msg = e.to_string();
            let (status, error_code, description) = if error_msg.contains("username_already_taken")
            {
                (
                    StatusCode::CONFLICT,
                    "username_unavailable",
                    "Username is already taken. Please choose a different username.".to_string(),
                )
            } else if error_msg.contains("email_already_registered") {
                (
                    StatusCode::CONFLICT,
                    "email_exists",
                    "An account with this email already exists.".to_string(),
                )
            } else if error_msg.contains("Registration state not found") {
                (
                    StatusCode::BAD_REQUEST,
                    "expired_challenge",
                    "Registration challenge has expired. Please start the registration process \
                     again."
                        .to_string(),
                )
            } else if error_msg.contains("finish_passkey_registration")
                || error_msg.contains("verification")
                || error_msg.contains("attestation")
            {
                (
                    StatusCode::BAD_REQUEST,
                    "invalid_credential",
                    "WebAuthn verification failed. Please ensure your authenticator and browser \
                     are compatible."
                        .to_string(),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "registration_failed",
                    format!("Registration failed: {error_msg}"),
                )
            };

            (
                status,
                Json(RegisterError {
                    error: error_code.to_string(),
                    error_description: description,
                }),
            )
                .into_response()
        },
    }
}
