use axum::extract::{Extension, Path, State};
use axum::response::{IntoResponse, Json};

use crate::models::clients::api::{OAuthClientResponse, UpdateOAuthClientRequest};
use crate::repository::OAuthRepository;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::api::{ApiError, SingleResponse};

pub async fn update_client(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    State(ctx): State<systemprompt_core_system::AppContext>,
    Path(client_id): Path<String>,
    Json(request): Json<UpdateOAuthClientRequest>,
) -> impl IntoResponse {
    let repository = OAuthRepository::new(ctx.db_pool().clone());
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());

    match repository.get_client(&client_id).await {
        Ok(Some(prev_client)) => {
            match repository
                .update_client(
                    &client_id,
                    request.name.as_deref(),
                    request.redirect_uris.as_deref(),
                    request.scopes.as_deref(),
                )
                .await
            {
                Ok(client) => {
                    logger.log(LogLevel::Info, "oauth_api", "OAuth client updated", Some(serde_json::json!({
                        "client_id": &client_id,
                        "client_name": &client.name,
                        "updated_by": req_ctx.auth.user_id.as_str(),
                        "changes": {
                            "name_changed": request.name.is_some() && request.name.as_deref() != prev_client.name.as_deref(),
                            "redirect_uris_changed": request.redirect_uris.is_some(),
                            "scopes_changed": request.scopes.is_some()
                        }
                    }))).await.ok();
                    let response: OAuthClientResponse = client.into();
                    SingleResponse::new(response).into_response()
                },
                Err(e) => {
                    logger
                        .log(
                            LogLevel::Error,
                            "oauth_api",
                            "OAuth client update failed",
                            Some(serde_json::json!({
                                "client_id": &client_id,
                                "reason": format!("Database error: {e}"),
                                "updated_by": req_ctx.auth.user_id.as_str()
                            })),
                        )
                        .await
                        .ok();
                    ApiError::bad_request(format!("Failed to update client: {e}")).into_response()
                },
            }
        },
        Ok(None) => {
            logger
                .log(
                    LogLevel::Info,
                    "oauth_api",
                    "OAuth client update failed",
                    Some(serde_json::json!({
                        "client_id": &client_id,
                        "reason": "Client not found",
                        "updated_by": req_ctx.auth.user_id.as_str()
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
                    "OAuth client update failed",
                    Some(serde_json::json!({
                        "client_id": &client_id,
                        "reason": format!("Database error: {e}"),
                        "updated_by": req_ctx.auth.user_id.as_str()
                    })),
                )
                .await
                .ok();
            ApiError::internal_error(format!("Failed to get client: {e}")).into_response()
        },
    }
}
