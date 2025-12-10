mod helpers;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::services::broadcaster::CONTEXT_BROADCASTER;
use systemprompt_core_system::AppContext;
use systemprompt_models::execution::events::BroadcastEvent;

use crate::repository::ContextRepository;

#[derive(Debug, Deserialize, Serialize)]
pub struct WebhookRequest {
    pub event_type: String,
    pub entity_id: String,
    pub context_id: String,
    pub user_id: String,
    #[serde(default)]
    pub step_data: Option<serde_json::Value>,
    #[serde(default)]
    pub task_data: Option<serde_json::Value>,
}

pub async fn broadcast_context_event(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    State(app_context): State<AppContext>,
    Json(request): Json<WebhookRequest>,
) -> Response {
    let start_time = std::time::Instant::now();
    let db = app_context.db_pool();
    let log_context = if matches!(
        request.event_type.as_str(),
        "task_completed" | "task_created"
    ) {
        req_ctx.log_context().with_task_id(&request.entity_id)
    } else {
        req_ctx.log_context()
    };
    let logger = LogService::new(db.clone(), log_context);

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

    let context_repo = ContextRepository::new(db.clone());
    if let Err(e) = context_repo
        .validate_context_ownership(&request.context_id, authenticated_user_id)
        .await
    {
        logger
            .log(
                LogLevel::Error,
                "webhook",
                &format!("Context ownership validation failed: {e}"),
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
                "message": format!("User does not own context: {e}")
            })),
        )
            .into_response();
    }

    logger
        .log(
            LogLevel::Debug,
            "webhook",
            &format!(
                "Webhook received | event={}, entity_id={}, context_id={}",
                request.event_type, request.entity_id, request.context_id
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

    let data = match helpers::load_event_data(&app_context, &request, &logger).await {
        Ok(data) => data,
        Err(e) => {
            logger
                .log(
                    LogLevel::Error,
                    "webhook",
                    &format!("Failed to load event data: {e}"),
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
            LogLevel::Debug,
            "webhook",
            &format!(
                "Webhook processed | event={}, broadcast_count={}",
                request.event_type, count
            ),
            Some(json!({
                "event_type": request.event_type,
                "connection_count": count,
                "user_id": request.user_id,
                "duration_ms": start_time.elapsed().as_millis(),
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
