use crate::repository::ContentRepository;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use systemprompt_core_system::{AppContext, RequestContext};

pub async fn list_content_by_source_handler(
    State(ctx): State<AppContext>,
    Path(source_id): Path<String>,
) -> impl IntoResponse {
    let content_repo = ContentRepository::new(ctx.db_pool().clone());

    match content_repo.list_by_source(&source_id).await {
        Ok(content) => Json(content).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_content_handler(
    State(ctx): State<AppContext>,
    Extension(_req_ctx): Extension<RequestContext>,
    Path((source_id, slug)): Path<(String, String)>,
) -> impl IntoResponse {
    let content_repo = ContentRepository::new(ctx.db_pool().clone());

    match content_repo.get_by_source_and_slug(&source_id, &slug).await {
        Ok(Some(content)) => Json(content).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Content not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
