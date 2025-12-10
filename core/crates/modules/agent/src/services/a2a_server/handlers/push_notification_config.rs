use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde_json::json;
use std::sync::Arc;

use crate::models::a2a::protocol::{
    DeleteTaskPushNotificationConfigRequest, GetTaskPushNotificationConfigRequest,
    SetTaskPushNotificationConfigRequest,
};
use crate::repository::PushNotificationConfigRepository;
use crate::services::a2a_server::handlers::AgentHandlerState;
use systemprompt_core_logging::LogService;

pub async fn handle_set_push_notification_config(
    State(state): State<Arc<AgentHandlerState>>,
    request: SetTaskPushNotificationConfigRequest,
    log: &LogService,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    log.info(
        "push_notification_config",
        &format!(
            "Setting push notification config for task: {}",
            request.task_id
        ),
    )
    .await
    .ok();

    let repo = PushNotificationConfigRepository::new(state.db_pool.clone());

    match repo.add_config(&request.task_id, &request.config).await {
        Ok(config_id) => {
            log.info(
                "push_notification_config",
                &format!(
                    "Successfully added config {} for task {}",
                    config_id, request.task_id
                ),
            )
            .await
            .ok();

            Ok((
                StatusCode::OK,
                Json(json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "success": true,
                        "configId": config_id,
                        "message": "Push notification config added successfully"
                    }
                })),
            ))
        },
        Err(e) => {
            log.error(
                "push_notification_config",
                &format!("Failed to add config for task {}: {}", request.task_id, e),
            )
            .await
            .ok();

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32603,
                        "message": "Failed to add push notification config",
                        "data": format!("{e}")
                    }
                })),
            ))
        },
    }
}

pub async fn handle_get_push_notification_config(
    State(state): State<Arc<AgentHandlerState>>,
    request: GetTaskPushNotificationConfigRequest,
    log: &LogService,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    log.info(
        "push_notification_config",
        &format!(
            "Getting push notification config for task: {}",
            request.task_id
        ),
    )
    .await
    .ok();

    let repo = PushNotificationConfigRepository::new(state.db_pool.clone());

    // A2A spec: if no specific config ID is provided, return all configs
    match repo.list_configs(&request.task_id).await {
        Ok(configs) => Ok((
            StatusCode::OK,
            Json(json!({
                "jsonrpc": "2.0",
                "result": {
                    "configs": configs
                }
            })),
        )),
        Err(e) => {
            log.error(
                "push_notification_config",
                &format!("Failed to get configs for task {}: {}", request.task_id, e),
            )
            .await
            .ok();

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32603,
                        "message": "Failed to get push notification configs",
                        "data": format!("{e}")
                    }
                })),
            ))
        },
    }
}

pub async fn handle_list_push_notification_configs(
    State(state): State<Arc<AgentHandlerState>>,
    task_id: String,
    log: &LogService,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    log.info(
        "push_notification_config",
        &format!("Listing push notification configs for task: {task_id}"),
    )
    .await
    .ok();

    let repo = PushNotificationConfigRepository::new(state.db_pool.clone());

    match repo.list_configs(&task_id).await {
        Ok(configs) => {
            let total = configs.len() as u32;

            Ok((
                StatusCode::OK,
                Json(json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "configs": configs,
                        "total": total
                    }
                })),
            ))
        },
        Err(e) => {
            log.error(
                "push_notification_config",
                &format!("Failed to list configs for task {task_id}: {e}"),
            )
            .await
            .ok();

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32603,
                        "message": "Failed to list push notification configs",
                        "data": format!("{e}")
                    }
                })),
            ))
        },
    }
}

pub async fn handle_delete_push_notification_config(
    State(state): State<Arc<AgentHandlerState>>,
    request: DeleteTaskPushNotificationConfigRequest,
    log: &LogService,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    log.info(
        "push_notification_config",
        &format!(
            "Deleting push notification config for task: {}",
            request.task_id
        ),
    )
    .await
    .ok();

    let repo = PushNotificationConfigRepository::new(state.db_pool.clone());

    // A2A spec: if no config_id provided in request, we need to extract it from
    // params For now, we'll delete all configs for the task as per simplified
    // implementation
    match repo.delete_all_for_task(&request.task_id).await {
        Ok(count) => {
            log.info(
                "push_notification_config",
                &format!(
                    "Successfully deleted {} configs for task {}",
                    count, request.task_id
                ),
            )
            .await
            .ok();

            Ok((
                StatusCode::OK,
                Json(json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "success": true,
                        "deleted": count,
                        "message": format!("Deleted {} push notification config(s)", count)
                    }
                })),
            ))
        },
        Err(e) => {
            log.error(
                "push_notification_config",
                &format!(
                    "Failed to delete configs for task {}: {}",
                    request.task_id, e
                ),
            )
            .await
            .ok();

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32603,
                        "message": "Failed to delete push notification configs",
                        "data": format!("{e}")
                    }
                })),
            ))
        },
    }
}
