mod helpers;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;

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

    let pool = match db.pool_arc() {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {e}")})),
            )
                .into_response();
        },
    };

    logger
        .debug(
            "context_notifications",
            &format!(
                "Received notification for context {}: {}",
                context_id, notification.method
            ),
        )
        .await
        .ok();

    let user_id = match sqlx::query_scalar::<_, String>(
        "SELECT user_id FROM user_contexts WHERE context_id = $1",
    )
    .bind(&context_id)
    .fetch_optional(pool.as_ref())
    .await
    {
        Ok(Some(uid)) => uid,
        Ok(None) => {
            logger
                .error(
                    "context_notifications",
                    &format!("Context {} not found", context_id),
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
        Err(e) => {
            logger
                .error(
                    "context_notifications",
                    &format!("Context {context_id} not found: {e}"),
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

    match helpers::persist_notification(db.clone(), &context_id, &agent_id, &notification).await {
        Ok(notification_id) => {
            logger
                .debug(
                    "context_notifications",
                    &format!(
                        "Persisted notification {} for context {}",
                        notification_id, context_id
                    ),
                )
                .await
                .ok();

            match helpers::process_notification(app_context.clone(), &notification).await {
                Ok(_) => {
                    match helpers::broadcast_notification(&context_id, &user_id, &notification)
                        .await
                    {
                        Ok(broadcast_count) => {
                            logger
                                .debug(
                                    "context_notifications",
                                    &format!(
                                        "Broadcasted notification to {} streams for context {}",
                                        broadcast_count, context_id
                                    ),
                                )
                                .await
                                .ok();

                            if let Err(e) =
                                helpers::mark_notification_broadcasted(db.clone(), notification_id)
                                    .await
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
                                    &format!("Failed to broadcast notification: {e}"),
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
                            &format!("Failed to process notification: {e}"),
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
                    &format!("Failed to persist notification: {e}"),
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
