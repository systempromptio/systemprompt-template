//! HTTP handlers for plugin listing and installation.

use axum::Json;
use axum::extract::{Extension, Query};
use axum::response::{IntoResponse, Response};

use crate::error::AdminResult;
use crate::handlers::shared;
use crate::repositories;
use crate::types::{UserContext, UserQuery};

use super::responses::PluginsListResponse;

pub(crate) async fn list_plugins_handler(
    Extension(user_ctx): Extension<UserContext>,
    Query(_query): Query<UserQuery>,
) -> AdminResult<Response> {
    let services_path = shared::get_services_path()?;
    let plugins = repositories::marketplace::plugins::list_plugins_for_roles(
        &services_path,
        &user_ctx.roles,
    )?;
    Ok(Json(PluginsListResponse { plugins }).into_response())
}
