use crate::models::SearchRequest;
use crate::services::SearchService;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;
use systemprompt_models::RequestContext;

pub async fn query_handler(
    Extension(req_ctx): Extension<RequestContext>,
    State(ctx): State<AppContext>,
    Json(request): Json<SearchRequest>,
) -> impl IntoResponse {
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());

    logger
        .info("rag_api", &format!("Searching for: {}", request.query))
        .await
        .ok();

    let search_service = SearchService::new(ctx.db_pool().clone());

    match search_service.search(&request).await {
        Ok(response) => {
            logger
                .info("rag_api", &format!("Found {} results", response.total))
                .await
                .ok();
            Json(response).into_response()
        },
        Err(e) => {
            logger
                .error("rag_api", &format!("Search error: {e}"))
                .await
                .ok();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        },
    }
}
