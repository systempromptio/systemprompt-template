use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::handlers::shared;
use crate::repositories;

use super::resources::get_services_path;

pub async fn get_plugin_detail_handler(Path(plugin_id): Path<String>) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::find_plugin_detail(&services_path, &plugin_id) {
        Ok(Some(plugin)) => Json(plugin).into_response(),
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "Plugin not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get plugin");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        }
    }
}
