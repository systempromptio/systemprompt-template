use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use serde::Deserialize;

use crate::models::a2a::TaskState;
use crate::repository::TaskRepository;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::{AppContext, RequestContext};

#[derive(Debug, Deserialize)]
pub struct TaskQueryParams {
    status: Option<String>,
    limit: Option<u32>,
}

pub async fn list_tasks_by_context(
    Extension(req_ctx): Extension<RequestContext>,
    State(app_context): State<AppContext>,
    Path(context_id): Path<String>,
) -> impl IntoResponse {
    let logger = LogService::new(app_context.db_pool().clone(), req_ctx.log_context());

    logger
        .info(
            "tasks_api",
            &format!("Listing tasks for context: {}", context_id),
        )
        .await
        .ok();

    let task_repo = TaskRepository::new(app_context.db_pool().clone());

    match task_repo.list_tasks_by_context(&context_id).await {
        Ok(tasks) => {
            logger
                .info(
                    "tasks_api",
                    &format!("Found {} tasks for context {}", tasks.len(), context_id),
                )
                .await
                .ok();
            (StatusCode::OK, Json(tasks)).into_response()
        },
        Err(e) => {
            logger
                .error("tasks_api", &format!("Failed to list tasks: {}", e))
                .await
                .ok();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve tasks",
                    "message": e.to_string()
                })),
            )
                .into_response()
        },
    }
}

pub async fn get_task(
    Extension(req_ctx): Extension<RequestContext>,
    State(app_context): State<AppContext>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let logger = LogService::new(app_context.db_pool().clone(), req_ctx.log_context());

    logger
        .info("tasks_api", &format!("Retrieving task: {}", task_id))
        .await
        .ok();

    let task_repo = TaskRepository::new(app_context.db_pool().clone());

    match task_repo.get_task(&task_id).await {
        Ok(Some(task)) => {
            logger
                .info("tasks_api", "Task retrieved successfully")
                .await
                .ok();
            (StatusCode::OK, Json(task)).into_response()
        },
        Ok(None) => {
            logger.info("tasks_api", "Task not found").await.ok();
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Task not found",
                    "task_id": task_id
                })),
            )
                .into_response()
        },
        Err(e) => {
            logger
                .error("tasks_api", &format!("Failed to retrieve task: {}", e))
                .await
                .ok();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve task",
                    "message": e.to_string()
                })),
            )
                .into_response()
        },
    }
}

pub async fn list_tasks_by_user(
    Extension(req_ctx): Extension<RequestContext>,
    State(app_context): State<AppContext>,
    Query(params): Query<TaskQueryParams>,
) -> impl IntoResponse {
    let logger = LogService::new(app_context.db_pool().clone(), req_ctx.log_context());

    let user_id = req_ctx.auth.user_id.as_str();

    logger
        .info("tasks_api", &format!("Listing tasks for user: {}", user_id))
        .await
        .ok();

    let task_repo = TaskRepository::new(app_context.db_pool().clone());

    let task_state = params.status.as_ref().and_then(|s| match s.as_str() {
        "submitted" => Some(TaskState::Submitted),
        "working" => Some(TaskState::Working),
        "input-required" => Some(TaskState::InputRequired),
        "completed" => Some(TaskState::Completed),
        "canceled" => Some(TaskState::Canceled),
        "cancelled" => Some(TaskState::Canceled),
        "failed" => Some(TaskState::Failed),
        "rejected" => Some(TaskState::Rejected),
        "auth-required" => Some(TaskState::AuthRequired),
        _ => None,
    });

    match task_repo
        .get_tasks_by_user_id(user_id, params.limit.map(|l| l as i32), None)
        .await
    {
        Ok(mut tasks) => {
            if let Some(state) = task_state {
                tasks.retain(|t| t.status.state == state);
            }

            logger
                .info(
                    "tasks_api",
                    &format!("Found {} tasks for user {}", tasks.len(), user_id),
                )
                .await
                .ok();
            (StatusCode::OK, Json(tasks)).into_response()
        },
        Err(e) => {
            logger
                .error("tasks_api", &format!("Failed to list tasks: {}", e))
                .await
                .ok();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve tasks",
                    "message": e.to_string()
                })),
            )
                .into_response()
        },
    }
}

pub async fn get_messages_by_task(
    Extension(req_ctx): Extension<RequestContext>,
    State(app_context): State<AppContext>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let logger = LogService::new(app_context.db_pool().clone(), req_ctx.log_context());

    logger
        .info(
            "tasks_api",
            &format!("Retrieving messages for task: {}", task_id),
        )
        .await
        .ok();

    let task_repo = TaskRepository::new(app_context.db_pool().clone());

    match task_repo.get_messages_by_task(&task_id).await {
        Ok(messages) => {
            logger
                .info(
                    "tasks_api",
                    &format!("Found {} messages for task {}", messages.len(), task_id),
                )
                .await
                .ok();
            (StatusCode::OK, Json(messages)).into_response()
        },
        Err(e) => {
            logger
                .error("tasks_api", &format!("Failed to retrieve messages: {}", e))
                .await
                .ok();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve messages",
                    "message": e.to_string()
                })),
            )
                .into_response()
        },
    }
}
