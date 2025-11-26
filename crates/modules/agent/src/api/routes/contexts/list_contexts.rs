use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};

use crate::repository::{ArtifactRepository, ContextRepository, TaskRepository};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::api::ApiError;

pub async fn list_contexts(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    State(ctx): State<systemprompt_core_system::AppContext>,
) -> impl IntoResponse {
    let db_pool = ctx.db_pool().clone();
    let task_repo = std::sync::Arc::new(TaskRepository::new(db_pool.clone()));
    let artifact_repo = std::sync::Arc::new(ArtifactRepository::new(db_pool.clone()));
    let context_repo = ContextRepository::new(db_pool.clone(), task_repo, artifact_repo);

    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());

    let user_id = &req_ctx.auth.user_id;

    match context_repo
        .list_contexts_with_stats(user_id.as_str())
        .await
    {
        Ok(contexts) => {
            logger
                .info(
                    "context_api",
                    &format!("Listed {} contexts for user {}", contexts.len(), user_id),
                )
                .await
                .ok();
            (StatusCode::OK, Json(contexts)).into_response()
        },
        Err(e) => {
            logger
                .error("context_api", &format!("Failed to list contexts: {}", e))
                .await
                .ok();
            ApiError::internal_error(format!("Failed to list contexts: {}", e)).into_response()
        },
    }
}
