use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::api::BlogState;
use crate::models::SearchRequest;
use crate::services::{ContentService, SearchService};

pub async fn query_handler(
    State(state): State<BlogState>,
    Json(request): Json<SearchRequest>,
) -> Response {
    tracing::info!(query = %request.query, "Searching content");

    let search_service = SearchService::new(state.pool.clone());

    match search_service.search(&request).await {
        Ok(response) => {
            tracing::info!(total = response.total, "Search completed");
            Json(response).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Search error");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn list_content_handler(
    State(state): State<BlogState>,
    Path(source_id): Path<String>,
) -> Response {
    let content_service = ContentService::new(state.pool.clone());

    match content_service.list_by_source(&source_id).await {
        Ok(content) => Json(content).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn get_content_handler(
    State(state): State<BlogState>,
    Path((source_id, slug)): Path<(String, String)>,
) -> Response {
    let content_service = ContentService::new(state.pool.clone());

    match content_service
        .get_by_source_and_slug(&source_id, &slug)
        .await
    {
        Ok(Some(content)) => Json(content).into_response(),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "Content not found"),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

fn error_response(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({"error": message}))).into_response()
}
