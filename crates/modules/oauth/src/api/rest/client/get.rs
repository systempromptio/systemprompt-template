use axum::{
    extract::{Extension, Path, State},
    response::IntoResponse,
};

use crate::models::clients::api::OAuthClientResponse;
use crate::repository::OAuthRepository;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::api::{ApiError, SingleResponse};

pub async fn get_client(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    State(ctx): State<systemprompt_core_system::AppContext>,
    Path(client_id): Path<String>,
) -> impl IntoResponse {
    let repository = OAuthRepository::new(ctx.db_pool().clone());
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());

    match repository.get_client(&client_id).await {
        Ok(Some(client)) => {
            logger
                .log(
                    LogLevel::Info,
                    "oauth_api",
                    "OAuth client retrieved",
                    Some(serde_json::json!({
                        "client_id": &client_id,
                        "client_name": &client.name,
                        "requested_by": req_ctx.auth.user_id.as_str()
                    })),
                )
                .await
                .ok();
            let response: OAuthClientResponse = client.into();
            SingleResponse::new(response).into_response()
        },
        Ok(None) => {
            logger
                .log(
                    LogLevel::Info,
                    "oauth_api",
                    "OAuth client retrieval failed",
                    Some(serde_json::json!({
                        "client_id": &client_id,
                        "reason": "Client not found",
                        "requested_by": req_ctx.auth.user_id.as_str()
                    })),
                )
                .await
                .ok();
            ApiError::not_found(format!("Client with ID '{client_id}' not found")).into_response()
        },
        Err(e) => {
            logger
                .log(
                    LogLevel::Error,
                    "oauth_api",
                    "OAuth client retrieval failed",
                    Some(serde_json::json!({
                        "client_id": &client_id,
                        "reason": format!("Database error: {}", e),
                        "requested_by": req_ctx.auth.user_id.as_str()
                    })),
                )
                .await
                .ok();
            ApiError::internal_error(format!("Failed to get client: {e}")).into_response()
        },
    }
}
