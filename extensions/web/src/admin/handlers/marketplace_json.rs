use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use systemprompt::identifiers::UserId;

use crate::admin::handlers::shared;
use crate::admin::repositories;

pub async fn marketplace_json_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id_raw): Path<String>,
    _headers: HeaderMap,
) -> Response {
    let user_id_str = user_id_raw
        .strip_suffix(".git")
        .unwrap_or(&user_id_raw)
        .to_string();
    let user_id = UserId::new(&user_id_str);

    if let Some(resp) = try_serve_persistent_json(&user_id) {
        return resp;
    }

    generate_marketplace_json(&pool, &user_id).await
}

fn try_serve_persistent_json(user_id: &UserId) -> Option<Response> {
    let persistent_path = std::path::PathBuf::from("storage/marketplace-versions")
        .join(user_id.as_str())
        .join("marketplace.json");
    if !persistent_path.is_file() {
        return None;
    }
    let content = match std::fs::read_to_string(&persistent_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to read persistent marketplace.json, falling back to on-demand generation");
            return None;
        }
    };
    match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(json) => Some(Json(json).into_response()),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to parse persistent marketplace.json, falling back to on-demand generation");
            None
        }
    }
}

async fn generate_marketplace_json(pool: &PgPool, user_id: &UserId) -> Response {
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let user_info = match repositories::marketplace_git::lookup_user_basic(pool, user_id).await {
        Ok(info) => info,
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "User not found for marketplace");
            return shared::error_response(StatusCode::NOT_FOUND, "User not found");
        }
    };

    if let Err(e) = repositories::marketplace_sync_status::mark_user_dirty(pool, user_id).await {
        tracing::warn!(error = %e, "Failed to mark user dirty");
    }

    let export_params = repositories::ExportParams {
        services_path: &services_path,
        pool,
        user_id,
        username: &user_info.display_name,
        email: &user_info.email,
        roles: &user_info.roles,
    };
    let response = match repositories::generate_export_bundles(&export_params).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "Failed to generate export bundles");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Export failed: {e}"),
            );
        }
    };

    let marketplace_json: serde_json::Value =
        match serde_json::from_str(&response.marketplace.content) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "Failed to parse marketplace JSON");
                return shared::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Invalid marketplace JSON",
                );
            }
        };

    Json(marketplace_json).into_response()
}
