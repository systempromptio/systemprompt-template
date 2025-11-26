use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde_json::json;

use crate::models::context::UpdateContextRequest;
use crate::repository::{ArtifactRepository, ContextRepository, TaskRepository};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::api::ApiError;
use systemprompt_core_system::BroadcastEvent;

pub async fn update_context(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    State(ctx): State<systemprompt_core_system::AppContext>,
    Path(context_id): Path<String>,
    Json(request): Json<UpdateContextRequest>,
) -> impl IntoResponse {
    let db_pool = ctx.db_pool().clone();
    let task_repo = std::sync::Arc::new(TaskRepository::new(db_pool.clone()));
    let artifact_repo = std::sync::Arc::new(ArtifactRepository::new(db_pool.clone()));
    let context_repo = ContextRepository::new(db_pool.clone(), task_repo, artifact_repo);
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());
    let user_id = &req_ctx.auth.user_id;

    match context_repo
        .update_context_name(&context_id, user_id.as_str(), &request.name)
        .await
    {
        Ok(()) => {
            logger
                .info(
                    "context_api",
                    &format!("Updated context {} for user {}", context_id, user_id),
                )
                .await
                .ok();

            match context_repo
                .get_context(&context_id, user_id.as_str())
                .await
            {
                Ok(context) => {
                    let stream_event = BroadcastEvent {
                        event_type: "context_updated".to_string(),
                        context_id: context_id.to_string(),
                        user_id: user_id.to_string(),
                        data: json!({
                            "name": request.name,
                        }),
                        timestamp: Utc::now(),
                    };

                    systemprompt_core_system::CONTEXT_BROADCASTER
                        .broadcast_to_user(user_id.as_str(), stream_event)
                        .await;

                    (StatusCode::OK, Json(context)).into_response()
                },
                Err(e) => {
                    logger
                        .error(
                            "context_api",
                            &format!("Failed to retrieve updated context: {}", e),
                        )
                        .await
                        .ok();
                    ApiError::internal_error(format!(
                        "Context updated but failed to retrieve: {}",
                        e
                    ))
                    .into_response()
                },
            }
        },
        Err(e) => {
            logger
                .error("context_api", &format!("Failed to update context: {}", e))
                .await
                .ok();
            ApiError::not_found(format!("Failed to update context: {}", e)).into_response()
        },
    }
}
