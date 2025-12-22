//! Link tracking API handlers.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use sqlx::PgPool;

use crate::api::{BlogState, ContentJourneyQuery, GenerateLinkRequest, GenerateLinkResponse};
use crate::services::{LinkAnalyticsService, LinkGenerationService, LinkService};

/// Generate a new tracking link.
pub async fn generate_link_handler(
    State(state): State<BlogState>,
    Json(request): Json<GenerateLinkRequest>,
) -> Response {
    let service = LinkGenerationService::new(state.pool.clone());

    match service
        .generate(request.target_url, request.campaign_name, request.utm_params)
        .await
    {
        Ok(link) => {
            let base_url = state
                .config
                .as_ref()
                .map(|c| c.base_url().as_str())
                .unwrap_or("https://example.com");
            let response = GenerateLinkResponse {
                id: link.id.to_string(),
                short_code: link.short_code.clone(),
                short_url: format!("{}/r/{}", base_url.trim_end_matches('/'), link.short_code),
                target_url: link.target_url,
            };
            Json(response).into_response()
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// List all links.
pub async fn list_links_handler(State(_state): State<BlogState>) -> Response {
    // List functionality not yet implemented
    Json(serde_json::json!({"links": [], "message": "List functionality not yet implemented"})).into_response()
}

/// Get link performance metrics.
pub async fn link_performance_handler(
    State(state): State<BlogState>,
    Path(link_id): Path<String>,
) -> Response {
    let service = LinkService::new(state.pool.clone());

    match service.get_performance(&link_id).await {
        Ok(Some(perf)) => Json(perf).into_response(),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "Link not found"),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// Get clicks for a link.
pub async fn link_clicks_handler(
    State(state): State<BlogState>,
    Path(link_id): Path<String>,
) -> Response {
    let service = LinkService::new(state.pool.clone());

    match service.get_clicks(&link_id, 100).await {
        Ok(clicks) => Json(clicks).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// Get campaign performance.
pub async fn campaign_performance_handler(
    State(state): State<BlogState>,
    Path(campaign_id): Path<String>,
) -> Response {
    let service = LinkAnalyticsService::new(state.pool.clone());

    match service.get_campaign_performance(&campaign_id).await {
        Ok(Some(perf)) => Json(perf).into_response(),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "Campaign not found"),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// Get content journey analytics.
pub async fn content_journey_handler(
    State(state): State<BlogState>,
    Query(query): Query<ContentJourneyQuery>,
) -> Response {
    let service = LinkAnalyticsService::new(state.pool.clone());

    match service.get_content_journey(&query.content_id).await {
        Ok(journey) => Json(journey).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// Handle redirect from short code.
pub async fn redirect_handler(
    State(pool): State<Arc<PgPool>>,
    Path(short_code): Path<String>,
) -> Response {
    let service = LinkService::new(pool);

    // Generate a session ID for tracking
    let session_id = uuid::Uuid::new_v4().to_string();

    match service
        .process_redirect(&short_code, &session_id, None, None)
        .await
    {
        Ok(target_url) => Redirect::temporary(&target_url).into_response(),
        Err(e) => {
            tracing::warn!(short_code = %short_code, error = %e, "Redirect failed");
            error_response(StatusCode::NOT_FOUND, "Link not found")
        }
    }
}

fn error_response(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({"error": message}))).into_response()
}
