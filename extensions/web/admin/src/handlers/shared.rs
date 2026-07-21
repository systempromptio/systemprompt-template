//! Helpers shared across admin handlers.

use std::path::PathBuf;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use systemprompt::config::ProfileBootstrap;

#[derive(Debug, Serialize)]
pub(crate) struct ErrorBody {
    pub error: String,
}

pub(crate) fn error_response(status: StatusCode, message: &str) -> Response {
    (
        status,
        Json(ErrorBody {
            error: message.to_owned(),
        }),
    )
        .into_response()
}

pub(crate) fn boxed_error_response(status: StatusCode, message: &str) -> Box<Response> {
    Box::new(error_response(status, message))
}

pub(crate) fn get_services_path() -> Result<PathBuf, Box<Response>> {
    ProfileBootstrap::get()
        .map(|p| PathBuf::from(&p.paths.services))
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to get profile bootstrap");
            boxed_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to load profile")
        })
}

pub(crate) fn get_profile_path() -> Result<PathBuf, Box<Response>> {
    ProfileBootstrap::get_path()
        .map(PathBuf::from)
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to get profile path");
            boxed_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load profile path",
            )
        })
}
