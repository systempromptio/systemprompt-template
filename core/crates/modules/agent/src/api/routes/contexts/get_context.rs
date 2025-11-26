use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};

use crate::repository::{ArtifactRepository, ContextRepository, TaskRepository};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::api::ApiError;

pub async fn get_context(
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
        .get_context(&context_id, user_id.as_str())
        .await
    {
        Ok(context) => {
            logger
                .info(
                    "context_api",
                    &format!("Retrieved context {} for user {}", context_id, user_id),
                )
                .await
                .ok();
            (StatusCode::OK, Json(context)).into_response()
        },
        Err(e) => {
            logger
                .error("context_api", &format!("Failed to get context: {}", e))
                .await
                .ok();
            ApiError::not_found(format!("Context not found: {}", e)).into_response()
        },
    }
}
