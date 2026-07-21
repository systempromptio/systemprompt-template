//! HTTP handlers for plugin listing and installation.

use axum::Json;
use axum::extract::{Extension, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::handlers::shared;
use crate::repositories;
use crate::types::{UserContext, UserQuery};

use super::responses::PluginsListResponse;

pub(crate) async fn list_plugins_handler(
    Extension(user_ctx): Extension<UserContext>,
    Query(_query): Query<UserQuery>,
) -> Response {
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let plugins = match repositories::marketplace::plugins::list_plugins_for_roles(
        &services_path,
        &user_ctx.roles,
    ) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list plugins");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            );
        },
    };

    Json(PluginsListResponse { plugins }).into_response()
}
