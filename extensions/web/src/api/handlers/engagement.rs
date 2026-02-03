use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use systemprompt::identifiers::{SessionId, UserId};

use crate::api::BlogState;
use crate::models::engagement::{
    BatchEngagementRequest, EngagementEventRequest, EngagementEventResponse,
};
use crate::repository::EngagementRepository;

pub async fn engagement_handler(
    State(state): State<BlogState>,
    Json(request): Json<EngagementEventRequest>,
) -> Response {
    let repo = EngagementRepository::new(state.pool.clone());

    let session_id = SessionId::new(
        request
            .session_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
    );
    let user_id = UserId::new(
        request
            .user_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
    );

    match repo.create_event(&session_id, &user_id, &request).await {
        Ok(event) => {
            let response = EngagementEventResponse {
                id: event.id.to_string(),
                page_url: event.page_url,
                created_at: event.created_at,
            };
            Json(response).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to record engagement event");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to record event")
        }
    }
}

pub async fn engagement_batch_handler(
    State(state): State<BlogState>,
    Json(request): Json<BatchEngagementRequest>,
) -> Response {
    let repo = EngagementRepository::new(state.pool.clone());

    let session_id = SessionId::new(
        request
            .session_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
    );
    let user_id = UserId::new(
        request
            .user_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
    );

    match repo
        .create_events_batch(&session_id, &user_id, &request.events)
        .await
    {
        Ok(events) => {
            let responses: Vec<EngagementEventResponse> = events
                .into_iter()
                .map(|event| EngagementEventResponse {
                    id: event.id.to_string(),
                    page_url: event.page_url,
                    created_at: event.created_at,
                })
                .collect();
            Json(serde_json::json!({
                "events": responses,
                "count": responses.len()
            }))
            .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to record batch engagement events");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to record events")
        }
    }
}

fn error_response(status: StatusCode, message: &str) -> Response {
    (
        status,
        Json(serde_json::json!({
            "error": message
        })),
    )
        .into_response()
}
