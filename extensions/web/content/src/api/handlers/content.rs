use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use systemprompt::identifiers::SourceId;

use crate::api::{BlogState, ErrorResponse};
use crate::services::{ContentService, SearchService};
use systemprompt_web_shared::models::SearchRequest;

pub async fn query_handler(
    State(state): State<BlogState>,
    Json(request): Json<SearchRequest>,
) -> Response {
    tracing::info!(query = %request.query, "Searching content");

    let search_service = SearchService::new(Arc::clone(&state.pool));

    match search_service.search(&request).await {
        Ok(response) => {
            tracing::info!(total = response.total, "Search completed");
            Json(response).into_response()
        },
        Err(e) => {
            tracing::error!(error = %e, "Search error");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        },
    }
}

pub async fn list_content_handler(
    State(state): State<BlogState>,
    Path(source_id): Path<SourceId>,
) -> Response {
    let content_service = ContentService::new(Arc::clone(&state.pool));

    match content_service.list_by_source(&source_id).await {
        Ok(content) => Json(content).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list content");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        },
    }
}

pub async fn get_content_handler(
    State(state): State<BlogState>,
    Path((source_id, slug)): Path<(SourceId, String)>,
) -> Response {
    let content_service = ContentService::new(Arc::clone(&state.pool));

    match content_service
        .get_by_source_and_slug(&source_id, &slug)
        .await
    {
        Ok(Some(content)) => Json(content).into_response(),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "Content not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get content");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        },
    }
}

fn error_response(status: StatusCode, message: &str) -> Response {
    (status, Json(ErrorResponse::new(message))).into_response()
}
