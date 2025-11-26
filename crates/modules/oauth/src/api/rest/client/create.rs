use axum::{
    extract::{Extension, State},
    response::{IntoResponse, Json},
};
use bcrypt::{hash, DEFAULT_COST};
use uuid::Uuid;

use crate::models::clients::api::{CreateOAuthClientRequest, OAuthClientResponse};
use crate::repository::OAuthRepository;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::api::{ApiError, CreatedResponse};

pub async fn create_client(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    State(ctx): State<systemprompt_core_system::AppContext>,
    Json(request): Json<CreateOAuthClientRequest>,
) -> impl IntoResponse {
    let repository = OAuthRepository::new(ctx.db_pool().clone());
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());

    let client_secret = Uuid::new_v4().to_string();
    let client_secret_hash = match hash(&client_secret, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            logger
                .log(
                    LogLevel::Error,
                    "oauth_api",
                    "OAuth client creation failed",
                    Some(serde_json::json!({
                        "client_id": request.client_id,
                        "reason": format!("Failed to hash secret: {}", e),
                        "created_by": req_ctx.auth.user_id.as_str()
                    })),
                )
                .await
                .ok();
            return ApiError::internal_error(format!("Failed to hash client secret: {e}"))
                .into_response();
        },
    };

    let default_grant_types = vec![
        "authorization_code".to_string(),
        "refresh_token".to_string(),
    ];
    let default_response_types = vec!["code".to_string()];

    match repository
        .create_client(
            &request.client_id,
            &client_secret_hash,
            &request.name,
            &request.redirect_uris,
            Some(&default_grant_types),
            Some(&default_response_types),
            &request.scopes,
            Some("client_secret_basic"),
            None,
            None,
            None,
        )
        .await
    {
        Ok(client) => {
            logger
                .log(
                    LogLevel::Info,
                    "oauth_api",
                    "OAuth client created",
                    Some(serde_json::json!({
                        "client_id": client.client_id,
                        "client_name": client.name,
                        "redirect_uris": request.redirect_uris,
                        "scopes": request.scopes,
                        "created_by": req_ctx.auth.user_id.as_str()
                    })),
                )
                .await
                .ok();

            let location = format!("/api/v1/core/oauth/clients/{}", client.client_id);
            let response: OAuthClientResponse = client.into();

            match serde_json::to_value(response) {
                Ok(mut response_json) => {
                    response_json["client_secret"] = serde_json::Value::String(client_secret);
                    CreatedResponse::new(response_json, location).into_response()
                },
                Err(e) => ApiError::internal_error(format!("Failed to serialize response: {e}"))
                    .into_response(),
            }
        },
        Err(e) => {
            let error_msg = format!("Failed to create client: {e}");
            let is_duplicate = error_msg.contains("UNIQUE constraint failed");

            logger.log(LogLevel::Info, "oauth_api", "OAuth client creation rejected", Some(serde_json::json!({
                "client_id": request.client_id,
                "reason": if is_duplicate { "Client ID already exists" } else { &error_msg },
                "created_by": req_ctx.auth.user_id.as_str()
            }))).await.ok();

            if is_duplicate {
                ApiError::conflict("Client with this ID already exists").into_response()
            } else {
                ApiError::bad_request(error_msg).into_response()
            }
        },
    }
}
