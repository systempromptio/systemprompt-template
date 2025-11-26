use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde_json::json;

use crate::repository::{ArtifactRepository, ContextRepository, TaskRepository};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::api::ApiError;
use systemprompt_core_system::BroadcastEvent;

pub async fn delete_context(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    State(ctx): State<systemprompt_core_system::AppContext>,
    Path(context_id): Path<String>,
) -> impl IntoResponse {
    let db_pool = ctx.db_pool().clone();
    let task_repo = std::sync::Arc::new(TaskRepository::new(db_pool.clone()));
    let artifact_repo = std::sync::Arc::new(ArtifactRepository::new(db_pool.clone()));
    let context_repo = ContextRepository::new(db_pool.clone(), task_repo, artifact_repo);
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());
    let user_id = &req_ctx.auth.user_id;

    match context_repo
        .delete_context(&context_id, user_id.as_str())
        .await
    {
        Ok(()) => {
            logger
                .info(
                    "context_api",
                    &format!("Deleted context {} for user {}", context_id, user_id),
                )
                .await
                .ok();

            let stream_event = BroadcastEvent {
                event_type: "context_deleted".to_string(),
                context_id: context_id.to_string(),
                user_id: user_id.to_string(),
                data: json!({}),
                timestamp: Utc::now(),
            };

            systemprompt_core_system::CONTEXT_BROADCASTER
                .broadcast_to_user(user_id.as_str(), stream_event)
                .await;

            StatusCode::NO_CONTENT.into_response()
        },
        Err(e) => {
            logger
                .error("context_api", &format!("Failed to delete context: {}", e))
                .await
                .ok();
            ApiError::not_found(format!("Failed to delete context: {}", e)).into_response()
        },
    }
}
