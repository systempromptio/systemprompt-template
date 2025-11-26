use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use systemprompt_core_database::DatabaseProvider;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::{AppContext, BroadcastEvent};

#[derive(Debug, Deserialize, Serialize)]
pub struct A2aNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

pub async fn handle_context_notification(
    Path(context_id): Path<String>,
    State(app_context): State<AppContext>,
    Json(notification): Json<A2aNotification>,
) -> Response {
    let db = app_context.db_pool();
    let logger = LogService::system(db.clone());

    logger
        .info(
            "context_notifications",
            &format!(
                "Received notification for context {}: {}",
                context_id, notification.method
            ),
        )
        .await
        .ok();

    let query = systemprompt_core_database::DatabaseQueryEnum::GetUserByContext.get(db.as_ref());
    let user_id_row = match db.as_ref().fetch_one(&query, &[&context_id]).await {
        Ok(row) => row,
        Err(e) => {
            logger
                .error(
                    "context_notifications",
                    &format!("Context {} not found: {}", context_id, e),
                )
                .await
                .ok();
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "Context not found",
                    "context_id": context_id
                })),
            )
                .into_response();
        },
    };

    let user_id = user_id_row
        .get("user_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing user_id"))
        .unwrap()
        .to_string();

    if notification.jsonrpc != "2.0" {
        logger
            .error(
                "context_notifications",
                &format!("Invalid JSON-RPC version: {}", notification.jsonrpc),
            )
            .await
            .ok();

        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid JSON-RPC version, must be 2.0"})),
        )
            .into_response();
    }

    let agent_id = notification
        .params
        .get("agentId")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    match persist_notification(db.clone(), &context_id, &agent_id, &notification).await {
        Ok(notification_id) => {
            logger
                .info(
                    "context_notifications",
                    &format!(
                        "Persisted notification {} for context {}",
                        notification_id, context_id
                    ),
                )
                .await
                .ok();

            match process_notification(app_context.clone(), &notification).await {
                Ok(_) => {
                    match broadcast_notification(&context_id, &user_id, &notification).await {
                        Ok(broadcast_count) => {
                            logger
                                .info(
                                    "context_notifications",
                                    &format!(
                                        "Broadcasted notification to {} streams for context {}",
                                        broadcast_count, context_id
                                    ),
                                )
                                .await
                                .ok();

                            if let Err(e) =
                                mark_notification_broadcasted(db.clone(), notification_id).await
                            {
                                logger
                                    .error(
                                        "context_notifications",
                                        &format!(
                                            "Failed to mark notification as broadcasted: {}",
                                            e
                                        ),
                                    )
                                    .await
                                    .ok();
                            }
                        },
                        Err(e) => {
                            logger
                                .error(
                                    "context_notifications",
                                    &format!("Failed to broadcast notification: {}", e),
                                )
                                .await
                                .ok();
                        },
                    }

                    (
                        StatusCode::OK,
                        Json(json!({
                            "status": "received",
                            "notification_id": notification_id
                        })),
                    )
                        .into_response()
                },
                Err(e) => {
                    logger
                        .error(
                            "context_notifications",
                            &format!("Failed to process notification: {}", e),
                        )
                        .await
                        .ok();

                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "Failed to process notification",
                            "details": e.to_string()
                        })),
                    )
                        .into_response()
                },
            }
        },
        Err(e) => {
            logger
                .error(
                    "context_notifications",
                    &format!("Failed to persist notification: {}", e),
                )
                .await
                .ok();

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to persist notification",
                    "details": e.to_string()
                })),
            )
                .into_response()
        },
    }
}

async fn persist_notification(
    db: systemprompt_core_database::DbPool,
    context_id: &str,
    agent_id: &str,
    notification: &A2aNotification,
) -> Result<i64, anyhow::Error> {
    let notification_data = serde_json::to_string(notification)?;

    let insert_query =
        systemprompt_core_database::DatabaseQueryEnum::InsertContextNotification.get(db.as_ref());
    let result = db
        .as_ref()
        .execute(
            &insert_query,
            &[
                &context_id,
                &agent_id,
                &notification.method,
                &notification_data,
                &Utc::now(),
            ],
        )
        .await?;

    Ok(result as i64)
}

async fn process_notification(
    app_context: AppContext,
    notification: &A2aNotification,
) -> Result<(), anyhow::Error> {
    let db = app_context.db_pool();

    match notification.method.as_str() {
        "notifications/taskStatusUpdate" => {
            let task_id = notification
                .params
                .get("taskId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing taskId in notification"))?;

            let status = notification
                .params
                .get("status")
                .ok_or_else(|| anyhow::anyhow!("Missing status in notification"))?;

            let state = status
                .get("state")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing state in status"))?;

            let timestamp = status
                .get("timestamp")
                .and_then(|v| systemprompt_core_database::parse_database_datetime(v))
                .unwrap_or_else(Utc::now);

            let query_enum = if state == "completed" {
                systemprompt_core_database::DatabaseQueryEnum::UpdateTaskStatusCompleted
            } else {
                systemprompt_core_database::DatabaseQueryEnum::UpdateTaskStatus
            };

            let query = query_enum.get(db.as_ref());

            db.as_ref()
                .execute(&query, &[&state, &timestamp, &task_id])
                .await?;

            Ok(())
        },
        "notifications/artifactCreated" => Ok(()),
        "notifications/messageAdded" => Ok(()),
        _ => Ok(()),
    }
}

async fn broadcast_notification(
    context_id: &str,
    user_id: &str,
    notification: &A2aNotification,
) -> Result<usize, anyhow::Error> {
    let mut total_broadcasts = 0;

    match notification.method.as_str() {
        "notifications/taskStatusUpdate" => {
            let status_timestamp = notification
                .params
                .get("status")
                .and_then(|s| s.get("timestamp"))
                .and_then(|v| systemprompt_core_database::parse_database_datetime(v))
                .unwrap_or_else(Utc::now);

            let task_event = BroadcastEvent {
                event_type: "task_status_changed".to_string(),
                context_id: context_id.to_string(),
                user_id: user_id.to_string(),
                data: json!({
                    "task_id": notification.params.get("taskId"),
                    "status": notification.params.get("status"),
                    "task": notification.params.get("task"),
                }),
                timestamp: status_timestamp,
            };

            total_broadcasts += systemprompt_core_system::CONTEXT_BROADCASTER
                .broadcast_to_user(user_id, task_event)
                .await;
        },
        "notifications/artifactCreated" => {
            let artifact_event = BroadcastEvent {
                event_type: "artifact_created".to_string(),
                context_id: context_id.to_string(),
                user_id: user_id.to_string(),
                data: json!({
                    "artifact": notification.params.get("artifact"),
                    "task_id": notification.params.get("taskId"),
                }),
                timestamp: Utc::now(),
            };

            total_broadcasts += systemprompt_core_system::CONTEXT_BROADCASTER
                .broadcast_to_user(user_id, artifact_event)
                .await;
        },
        "notifications/messageAdded" => {
            let message_event = BroadcastEvent {
                event_type: "message_added".to_string(),
                context_id: context_id.to_string(),
                user_id: user_id.to_string(),
                data: json!({
                    "message_id": notification.params.get("messageId"),
                    "message": notification.params.get("message"),
                }),
                timestamp: Utc::now(),
            };

            total_broadcasts += systemprompt_core_system::CONTEXT_BROADCASTER
                .broadcast_to_user(user_id, message_event)
                .await;
        },
        _ => {},
    }

    Ok(total_broadcasts)
}

async fn mark_notification_broadcasted(
    db: systemprompt_core_database::DbPool,
    notification_id: i64,
) -> Result<(), anyhow::Error> {
    let update_query =
        systemprompt_core_database::DatabaseQueryEnum::MarkNotificationBroadcasted.get(db.as_ref());
    db.as_ref()
        .execute(&update_query, &[&notification_id])
        .await?;

    Ok(())
}
