//! HTTP handlers for plugin listing and installation.

use axum::Json;
use axum::extract::{Extension, Query};
use axum::response::{IntoResponse, Response};

use crate::error::{AdminError, AdminResult};
use crate::handlers::shared;
use crate::repositories;
use crate::types::{UserContext, UserQuery};

use super::responses::PluginsListResponse;

pub(crate) async fn list_plugins_handler(
    Extension(user_ctx): Extension<UserContext>,
    Query(_query): Query<UserQuery>,
) -> AdminResult<Response> {
    let services_path = shared::get_services_path()?;
    // Why not a bare `?`: this reads the plugin YAML off disk, so a
    // `MarketplaceError::NotFound` means a *server-side* file is missing, not
    // that the collection the client asked for does not exist. Propagating it
    // would answer 404 on a list endpoint that is always present.
    let plugins =
        repositories::marketplace::plugins::list_plugins_for_roles(&services_path, &user_ctx.roles)
            .map_err(AdminError::internal)?;
    Ok(Json(PluginsListResponse { plugins }).into_response())
}
