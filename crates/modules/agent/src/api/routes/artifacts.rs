use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::Extension;
use serde::Deserialize;

use crate::repository::ArtifactRepository;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::{AppContext, RequestContext};

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ArtifactQueryParams {
    limit: Option<u32>,
}

pub async fn list_artifacts_by_context(
    Extension(req_ctx): Extension<RequestContext>,
    State(app_context): State<AppContext>,
    Path(context_id): Path<String>,
) -> impl IntoResponse {
    let logger = LogService::new(app_context.db_pool().clone(), req_ctx.log_context());

    logger
        .debug(
            "artifacts_api",
            &format!("Listing artifacts | context_id={context_id}"),
        )
        .await
        .ok();

    let artifact_repo = ArtifactRepository::new(app_context.db_pool().clone());

    match artifact_repo.get_artifacts_by_context(&context_id).await {
        Ok(artifacts) => {
            logger
                .debug(
                    "artifacts_api",
                    &format!(
                        "Artifacts listed | context_id={}, count={}",
                        context_id,
                        artifacts.len()
                    ),
                )
                .await
                .ok();
            (StatusCode::OK, Json(artifacts)).into_response()
        },
        Err(e) => {
            logger
                .error("artifacts_api", &format!("Failed to list artifacts: {e}"))
                .await
                .ok();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve artifacts",
                    "message": e.to_string()
                })),
            )
                .into_response()
        },
    }
}

pub async fn list_artifacts_by_task(
    Extension(req_ctx): Extension<RequestContext>,
    State(app_context): State<AppContext>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let logger = LogService::new(app_context.db_pool().clone(), req_ctx.log_context());

    logger
        .debug(
            "artifacts_api",
            &format!("Listing artifacts | task_id={task_id}"),
        )
        .await
        .ok();

    let artifact_repo = ArtifactRepository::new(app_context.db_pool().clone());

    match artifact_repo.get_artifacts_by_task(&task_id).await {
        Ok(artifacts) => {
            logger
                .debug(
                    "artifacts_api",
                    &format!(
                        "Artifacts listed | task_id={}, count={}",
                        task_id,
                        artifacts.len()
                    ),
                )
                .await
                .ok();
            (StatusCode::OK, Json(artifacts)).into_response()
        },
        Err(e) => {
            logger
                .error("artifacts_api", &format!("Failed to list artifacts: {e}"))
                .await
                .ok();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve artifacts",
                    "message": e.to_string()
                })),
            )
                .into_response()
        },
    }
}

pub async fn get_artifact(
    Extension(req_ctx): Extension<RequestContext>,
    State(app_context): State<AppContext>,
    Path(artifact_id): Path<String>,
) -> impl IntoResponse {
    let logger = LogService::new(app_context.db_pool().clone(), req_ctx.log_context());

    logger
        .debug(
            "artifacts_api",
            &format!("Retrieving artifact | artifact_id={artifact_id}"),
        )
        .await
        .ok();

    let artifact_repo = ArtifactRepository::new(app_context.db_pool().clone());

    match artifact_repo.get_artifact_by_id(&artifact_id).await {
        Ok(Some(artifact)) => {
            logger
                .debug("artifacts_api", "Artifact retrieved successfully")
                .await
                .ok();
            (StatusCode::OK, Json(artifact)).into_response()
        },
        Ok(None) => {
            logger
                .debug("artifacts_api", "Artifact not found")
                .await
                .ok();
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Artifact not found",
                    "artifact_id": artifact_id
                })),
            )
                .into_response()
        },
        Err(e) => {
            logger
                .error(
                    "artifacts_api",
                    &format!("Failed to retrieve artifact: {e}"),
                )
                .await
                .ok();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve artifact",
                    "message": e.to_string()
                })),
            )
                .into_response()
        },
    }
}

pub async fn list_artifacts_by_user(
    Extension(req_ctx): Extension<RequestContext>,
    State(app_context): State<AppContext>,
    Query(params): Query<ArtifactQueryParams>,
) -> impl IntoResponse {
    let logger = LogService::new(app_context.db_pool().clone(), req_ctx.log_context());

    let user_id = req_ctx.auth.user_id.as_str();

    logger
        .debug(
            "artifacts_api",
            &format!("Listing artifacts | user_id={user_id}"),
        )
        .await
        .ok();

    let artifact_repo = ArtifactRepository::new(app_context.db_pool().clone());

    match artifact_repo
        .get_artifacts_by_user_id(user_id, params.limit.map(|l| l as i32))
        .await
    {
        Ok(artifacts) => {
            logger
                .debug(
                    "artifacts_api",
                    &format!(
                        "Artifacts listed | user_id={}, count={}",
                        user_id,
                        artifacts.len()
                    ),
                )
                .await
                .ok();
            (StatusCode::OK, Json(artifacts)).into_response()
        },
        Err(e) => {
            logger
                .error("artifacts_api", &format!("Failed to list artifacts: {e}"))
                .await
                .ok();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve artifacts",
                    "message": e.to_string()
                })),
            )
                .into_response()
        },
    }
}
