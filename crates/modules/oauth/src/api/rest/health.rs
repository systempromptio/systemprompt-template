use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde_json::json;

pub async fn handle_health_api(
    State(_ctx): State<systemprompt_core_system::AppContext>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "service": "oauth"
        })),
    )
}
