use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::handlers::shared;
use crate::repositories;
use crate::types::{GatewayRouteView, ReorderRoutesRequest, UpdateGatewaySettingsRequest};

pub async fn get_gateway_handler() -> Response {
    let profile_path = match shared::get_profile_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::get_gateway_config(&profile_path) {
        Ok(c) => Json(c).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get gateway config");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to load gateway")
        }
    }
}

pub async fn update_gateway_settings_handler(
    Json(body): Json<UpdateGatewaySettingsRequest>,
) -> Response {
    let profile_path = match shared::get_profile_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::update_gateway_settings(&profile_path, &body) {
        Ok(c) => Json(c).into_response(),
        Err(e) => map_error(e, "update gateway settings"),
    }
}

pub async fn create_gateway_route_handler(Json(body): Json<GatewayRouteView>) -> Response {
    let profile_path = match shared::get_profile_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::create_gateway_route(&profile_path, &body) {
        Ok(idx) => (StatusCode::CREATED, Json(serde_json::json!({"index": idx}))).into_response(),
        Err(e) => map_error(e, "create gateway route"),
    }
}

pub async fn update_gateway_route_handler(
    Path(idx): Path<usize>,
    Json(body): Json<GatewayRouteView>,
) -> Response {
    let profile_path = match shared::get_profile_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::update_gateway_route(&profile_path, idx, &body) {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "Route not found"),
        Err(e) => map_error(e, "update gateway route"),
    }
}

pub async fn delete_gateway_route_handler(Path(idx): Path<usize>) -> Response {
    let profile_path = match shared::get_profile_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::delete_gateway_route(&profile_path, idx) {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "Route not found"),
        Err(e) => map_error(e, "delete gateway route"),
    }
}

pub async fn reorder_gateway_routes_handler(Json(body): Json<ReorderRoutesRequest>) -> Response {
    let profile_path = match shared::get_profile_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::reorder_gateway_routes(&profile_path, &body.order) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => map_error(e, "reorder gateway routes"),
    }
}

fn map_error(err: systemprompt_web_shared::error::MarketplaceError, op: &str) -> Response {
    use systemprompt_web_shared::error::MarketplaceError as ME;
    match err {
        ME::BadRequest(msg) => shared::error_response(StatusCode::BAD_REQUEST, &msg),
        ME::NotFound(msg) => shared::error_response(StatusCode::NOT_FOUND, &msg),
        other => {
            tracing::error!(error = %other, op = op, "gateway operation failed");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        }
    }
}
