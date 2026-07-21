//! Helpers shared across admin handlers.

use std::path::PathBuf;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use systemprompt::config::ProfileBootstrap;

use crate::error::AdminResult;

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

pub(crate) fn get_services_path() -> AdminResult<PathBuf> {
    Ok(PathBuf::from(&ProfileBootstrap::get()?.paths.services))
}

pub(crate) fn get_profile_path() -> AdminResult<PathBuf> {
    Ok(PathBuf::from(ProfileBootstrap::get_path()?))
}
