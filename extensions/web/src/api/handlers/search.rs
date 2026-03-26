use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};

use crate::api::{BlogState, SearchQuery};
use crate::models::SearchRequest;
use crate::services::SearchService;

pub async fn search_handler(
    State(state): State<BlogState>,
    Query(query): Query<SearchQuery>,
) -> Response {
    let service = SearchService::new(state.pool.clone());

    let request = SearchRequest {
        query: query.q,
        filters: None,
        limit: query.limit,
    };

    match service.search(&request).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "Search failed");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
