//! HTTP handlers for gateway route configuration.

use axum::Json;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::error::{AdminError, AdminResult};
use crate::handlers::shared;
use crate::repositories;
use crate::types::{GatewayRouteView, ReorderRoutesRequest, UpdateGatewaySettingsRequest};

/// JSON body returned by [`create_gateway_route_handler`] on success.
#[derive(Debug, Serialize)]
pub(crate) struct CreateRouteResponse {
    pub index: usize,
}

pub(crate) async fn get_gateway_handler() -> AdminResult<Response> {
    let profile_path = shared::get_profile_path()?;
    let config = repositories::config::gateway::get_gateway_config(&profile_path)
        .map_err(AdminError::internal)?;
    Ok(Json(config).into_response())
}

pub(crate) async fn update_gateway_settings_handler(
    Json(body): Json<UpdateGatewaySettingsRequest>,
) -> AdminResult<Response> {
    let profile_path = shared::get_profile_path()?;
    let config = repositories::config::gateway::update_gateway_settings(&profile_path, &body)?;
    Ok(Json(config).into_response())
}

pub(crate) async fn create_gateway_route_handler(
    Json(body): Json<GatewayRouteView>,
) -> AdminResult<Response> {
    let profile_path = shared::get_profile_path()?;
    let index = repositories::config::gateway::create_route(&profile_path, &body)?;
    Ok((StatusCode::CREATED, Json(CreateRouteResponse { index })).into_response())
}

pub(crate) async fn update_gateway_route_handler(
    Path(idx): Path<usize>,
    Json(body): Json<GatewayRouteView>,
) -> AdminResult<Response> {
    let profile_path = shared::get_profile_path()?;
    if repositories::config::gateway::update_route(&profile_path, idx, &body)? {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Err(AdminError::NotFound("Route not found".to_owned()))
    }
}

pub(crate) async fn delete_gateway_route_handler(Path(idx): Path<usize>) -> AdminResult<Response> {
    let profile_path = shared::get_profile_path()?;
    if repositories::config::gateway::delete_route(&profile_path, idx)? {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Err(AdminError::NotFound("Route not found".to_owned()))
    }
}

pub(crate) async fn reorder_gateway_routes_handler(
    Json(body): Json<ReorderRoutesRequest>,
) -> AdminResult<Response> {
    let profile_path = shared::get_profile_path()?;
    repositories::config::gateway::reorder_routes(&profile_path, &body.order)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
