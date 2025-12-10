use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::repository::OAuthRepository;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::api::ApiError;

pub async fn delete_client(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    State(ctx): State<systemprompt_core_system::AppContext>,
    Path(client_id): Path<String>,
) -> impl IntoResponse {
    let repository = OAuthRepository::new(ctx.db_pool().clone());
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());

    match repository.get_client(&client_id).await {
        Ok(Some(client)) => match repository.delete_client(&client_id).await {
            Ok(_) => {
                logger
                    .log(
                        LogLevel::Info,
                        "oauth_api",
                        "OAuth client deleted",
                        Some(serde_json::json!({
                            "client_id": &client_id,
                            "client_name": client.name,
                            "deleted_by": req_ctx.auth.user_id.as_str()
                        })),
                    )
                    .await
                    .ok();
                StatusCode::NO_CONTENT.into_response()
            },
            Err(e) => {
                logger
                    .log(
                        LogLevel::Error,
                        "oauth_api",
                        "OAuth client deletion failed",
                        Some(serde_json::json!({
                            "client_id": &client_id,
                            "reason": format!("Database error: {e}"),
                            "deleted_by": req_ctx.auth.user_id.as_str()
                        })),
                    )
                    .await
                    .ok();
                ApiError::internal_error(format!("Failed to delete client: {e}")).into_response()
            },
        },
        Ok(None) => {
            logger
                .log(
                    LogLevel::Info,
                    "oauth_api",
                    "OAuth client deletion failed",
                    Some(serde_json::json!({
                        "client_id": &client_id,
                        "reason": "Client not found",
                        "deleted_by": req_ctx.auth.user_id.as_str()
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
                    "OAuth client deletion failed",
                    Some(serde_json::json!({
                        "client_id": &client_id,
                        "reason": format!("Database error: {e}"),
                        "deleted_by": req_ctx.auth.user_id.as_str()
                    })),
                )
                .await
                .ok();
            ApiError::internal_error(format!("Failed to get client: {e}")).into_response()
        },
    }
}
