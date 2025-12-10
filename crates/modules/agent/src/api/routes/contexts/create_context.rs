use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use serde_json::json;

use crate::models::context::CreateContextRequest;
use crate::repository::ContextRepository;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::api::ApiError;
use systemprompt_core_system::BroadcastEvent;

pub async fn create_context(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    State(ctx): State<systemprompt_core_system::AppContext>,
    Json(request): Json<CreateContextRequest>,
) -> impl IntoResponse {
    let db_pool = ctx.db_pool().clone();
    let context_repo = ContextRepository::new(db_pool.clone());
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());
    let user_id = &req_ctx.auth.user_id;

    match context_repo
        .create_context(
            user_id.as_str(),
            req_ctx.request.session_id.as_str(),
            &request.name,
        )
        .await
    {
        Ok(context_id) => {
            logger
                .debug(
                    "context_api",
                    &format!("Created context {context_id} for user {user_id}"),
                )
                .await
                .ok();

            match context_repo
                .get_context(&context_id, user_id.as_str())
                .await
            {
                Ok(context) => {
                    let stream_event = BroadcastEvent {
                        event_type: "context_created".to_string(),
                        context_id: context_id.to_string(),
                        user_id: user_id.to_string(),
                        data: json!({
                            "context": {
                                "context_id": &context.context_id,
                                "name": &context.name,
                                "created_at": &context.created_at,
                                "updated_at": &context.updated_at,
                            }
                        }),
                        timestamp: Utc::now(),
                    };

                    systemprompt_core_system::CONTEXT_BROADCASTER
                        .broadcast_to_user(user_id.as_str(), stream_event)
                        .await;

                    (StatusCode::CREATED, Json(context)).into_response()
                },
                Err(e) => {
                    logger
                        .error(
                            "context_api",
                            &format!("Failed to retrieve created context: {e}"),
                        )
                        .await
                        .ok();
                    ApiError::internal_error(format!(
                        "Context created but failed to retrieve: {}",
                        e
                    ))
                    .into_response()
                },
            }
        },
        Err(e) => {
            logger
                .error("context_api", &format!("Failed to create context: {e}"))
                .await
                .ok();
            ApiError::internal_error(format!("Failed to create context: {e}")).into_response()
        },
    }
}
