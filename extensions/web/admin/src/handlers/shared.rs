use std::path::PathBuf;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use systemprompt::models::ProfileBootstrap;

pub fn error_response(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({"error": message}))).into_response()
}

pub fn boxed_error_response(status: StatusCode, message: &str) -> Box<Response> {
    Box::new(error_response(status, message))
}

pub fn get_services_path() -> Result<PathBuf, Box<Response>> {
    ProfileBootstrap::get()
        .map(|p| PathBuf::from(&p.paths.services))
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to get profile bootstrap");
            boxed_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to load profile")
        })
}

pub fn normalize_user_id(raw: &str) -> &str {
    raw.strip_suffix(".git").unwrap_or(raw)
}
