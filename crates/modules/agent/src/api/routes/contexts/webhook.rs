use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use systemprompt_core_database::DatabaseProvider;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::services::broadcaster::CONTEXT_BROADCASTER;
use systemprompt_core_system::AppContext;
use systemprompt_models::execution::events::BroadcastEvent;

use crate::repository::{ArtifactRepository, ContextRepository, TaskRepository};

#[derive(Debug, Deserialize, Serialize)]
pub struct WebhookRequest {
    pub event_type: String,
    pub entity_id: String,
    pub context_id: String,
    pub user_id: String,
}

pub async fn broadcast_context_event(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    State(app_context): State<AppContext>,
    Json(request): Json<WebhookRequest>,
) -> Response {
    let db = app_context.db_pool();
    let logger = LogService::new(db.clone(), req_ctx.log_context());

    let authenticated_user_id = req_ctx.auth.user_id.as_str();

    if authenticated_user_id != request.user_id {
        logger
            .log(
                LogLevel::Error,
                "webhook",
                &format!(
                    "User mismatch: JWT user_id={}, payload user_id={}",
                    authenticated_user_id, request.user_id
                ),
                Some(json!({
                    "jwt_user_id": authenticated_user_id,
                    "payload_user_id": request.user_id,
                    "context_id": request.context_id,
                })),
            )
            .await
            .ok();

        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "User ID mismatch",
                "message": "Authenticated user does not match the request user_id"
            })),
        )
            .into_response();
    }

    let task_repo = std::sync::Arc::new(TaskRepository::new(db.clone()));
    let artifact_repo = std::sync::Arc::new(ArtifactRepository::new(db.clone()));
    let context_repo = ContextRepository::new(db.clone(), task_repo, artifact_repo);
    if let Err(e) = context_repo
        .validate_context_ownership(&request.context_id, authenticated_user_id)
        .await
    {
        logger
            .log(
                LogLevel::Error,
                "webhook",
                &format!("Context ownership validation failed: {}", e),
                Some(json!({
                    "context_id": request.context_id,
                    "user_id": authenticated_user_id,
                })),
            )
            .await
            .ok();

        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "Context ownership validation failed",
                "message": format!("User does not own context: {}", e)
            })),
        )
            .into_response();
    }

    logger
        .log(
            LogLevel::Info,
            "webhook",
            &format!(
                "Received webhook: event_type={}, entity_id={}, context_id={}, user_id={}",
                request.event_type, request.entity_id, request.context_id, request.user_id
            ),
            Some(json!({
                "event_type": request.event_type,
                "entity_id": request.entity_id,
                "context_id": request.context_id,
                "user_id": request.user_id,
            })),
        )
        .await
        .ok();

    let data = match load_event_data(&app_context, &request, &logger).await {
        Ok(data) => data,
        Err(e) => {
            logger
                .log(
                    LogLevel::Error,
                    "webhook",
                    &format!("Failed to load event data: {}", e),
                    Some(json!({
                        "error": e.to_string(),
                        "event_type": request.event_type,
                        "entity_id": request.entity_id,
                    })),
                )
                .await
                .ok();

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to load event data",
                    "details": e.to_string()
                })),
            )
                .into_response();
        },
    };

    let event = BroadcastEvent {
        event_type: request.event_type.clone(),
        context_id: request.context_id.clone(),
        user_id: request.user_id.clone(),
        data,
        timestamp: chrono::Utc::now(),
    };

    let count = CONTEXT_BROADCASTER
        .broadcast_to_user(&request.user_id, event)
        .await;

    logger
        .log(
            LogLevel::Info,
            "webhook",
            &format!(
                "Broadcast {} event to {} connections",
                request.event_type, count
            ),
            Some(json!({
                "event_type": request.event_type,
                "connection_count": count,
                "user_id": request.user_id,
            })),
        )
        .await
        .ok();

    (
        StatusCode::OK,
        Json(json!({
            "status": "broadcasted",
            "connection_count": count,
            "event_type": request.event_type
        })),
    )
        .into_response()
}

async fn load_event_data(
    app_context: &AppContext,
    request: &WebhookRequest,
    logger: &LogService,
) -> Result<serde_json::Value, anyhow::Error> {
    let db = app_context.db_pool();

    match request.event_type.as_str() {
        "task_completed" => {
            let task_repo = TaskRepository::new(db.clone());
            let artifact_repo = ArtifactRepository::new(db.clone());

            logger
                .info(
                    "webhook",
                    &format!("Completing task {} before broadcast", request.entity_id),
                )
                .await
                .ok();

            // Complete the task first
            use crate::models::a2a::TaskState;
            let timestamp = chrono::Utc::now();
            task_repo
                .update_task_state(&request.entity_id, TaskState::Completed, &timestamp)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to complete task: {}", e))?;

            logger
                .info(
                    "webhook",
                    &format!("Task {} marked as completed", request.entity_id),
                )
                .await
                .ok();

            // Now load task, artifacts, and messages for broadcast
            let task = task_repo
                .get_task(&request.entity_id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load task: {}", e))?;

            let artifacts = artifact_repo
                .get_artifacts_by_task(&request.entity_id)
                .await
                .unwrap_or_default();

            let messages = task_repo
                .get_messages_by_task(&request.entity_id)
                .await
                .unwrap_or_default();

            logger
                .info(
                    "webhook",
                    &format!(
                        "Broadcasting task {} with {} artifacts and {} messages",
                        request.entity_id,
                        artifacts.len(),
                        messages.len()
                    ),
                )
                .await
                .ok();

            let sanitized_data = json!({
                "task": task,
                "artifacts": artifacts,
                "messages": messages,
            });

            validate_json_serializable(&sanitized_data)
                .map_err(|e| anyhow::anyhow!("JSON validation failed: {}", e))?;

            Ok(sanitized_data)
        },
        "artifact_created" => {
            let artifact_repo = ArtifactRepository::new(db.clone());

            logger
                .info(
                    "webhook",
                    &format!("Loading artifact {}", request.entity_id),
                )
                .await
                .ok();

            let artifact = artifact_repo
                .get_artifact_by_id(&request.entity_id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load artifact: {}", e))?
                .ok_or_else(|| anyhow::anyhow!("Artifact not found: {}", request.entity_id))?;

            Ok(json!({
                "artifact": artifact,
            }))
        },
        "message_received" => {
            // Load message from database
            let query =
                systemprompt_core_database::DatabaseQueryEnum::GetMessageById.get(db.as_ref());
            let row = db
                .as_ref()
                .fetch_optional(&query, &[&request.entity_id])
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load message: {}", e))?;

            if let Some(row) = row {
                Ok(json!({
                    "message": row,
                }))
            } else {
                Err(anyhow::anyhow!("Message not found: {}", request.entity_id))
            }
        },
        "context_updated" => {
            let task_repo = std::sync::Arc::new(TaskRepository::new(db.clone()));
            let artifact_repo = std::sync::Arc::new(ArtifactRepository::new(db.clone()));
            let context_repo = ContextRepository::new(db.clone(), task_repo, artifact_repo);

            logger
                .info(
                    "webhook",
                    &format!("Loading context {}", request.context_id),
                )
                .await
                .ok();

            let context = context_repo
                .get_context(&request.context_id, &request.user_id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load context: {}", e))?;

            Ok(json!({
                "context": context,
            }))
        },
        _ => Err(anyhow::anyhow!(
            "Unknown event type: {}",
            request.event_type
        )),
    }
}

fn validate_json_serializable(value: &serde_json::Value) -> Result<(), String> {
    const MAX_PAYLOAD_SIZE: usize = 1_000_000;
    const MAX_TEXT_FIELD_SIZE: usize = 100_000;

    let sanitized = sanitize_payload(value, MAX_TEXT_FIELD_SIZE);

    let serialized = serde_json::to_string(&sanitized)
        .map_err(|e| format!("Failed to serialize to string: {}", e))?;

    if serialized.len() > MAX_PAYLOAD_SIZE {
        return Err(format!(
            "Payload too large: {} bytes (max: {})",
            serialized.len(),
            MAX_PAYLOAD_SIZE
        ));
    }

    serde_json::from_str::<serde_json::Value>(&serialized)
        .map_err(|e| format!("Re-parsing failed: {}", e))?;

    Ok(())
}

fn sanitize_payload(value: &serde_json::Value, max_text_size: usize) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => {
            if s.len() > max_text_size {
                serde_json::Value::String(format!(
                    "{}... [truncated from {} bytes]",
                    &s[..max_text_size.min(s.len())],
                    s.len()
                ))
            } else {
                serde_json::Value::String(s.clone())
            }
        },
        serde_json::Value::Array(arr) => serde_json::Value::Array(
            arr.iter()
                .map(|v| sanitize_payload(v, max_text_size))
                .collect(),
        ),
        serde_json::Value::Object(obj) => {
            let sanitized: serde_json::Map<String, serde_json::Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), sanitize_payload(v, max_text_size)))
                .collect();
            serde_json::Value::Object(sanitized)
        },
        other => other.clone(),
    }
}
